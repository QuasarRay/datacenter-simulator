// SPDX-License-Identifier: RPL-1.5
// Rust simulator extension of QuasarRay/datacenter-simulator.
// Unless explicitly acquired and licensed from Licensor under another license,
// the contents of this file are subject to the Reciprocal Public License
// ("RPL") Version 1.5, or subsequent versions as allowed by the RPL, and You
// may not copy or use this file in either source code or executable form,
// except in compliance with the terms and conditions of the RPL.
// All software distributed under the RPL is provided strictly on an "AS IS"
// basis, WITHOUT WARRANTY OF ANY KIND, EITHER EXPRESS OR IMPLIED, AND LICENSOR
// HEREBY DISCLAIMS ALL SUCH WARRANTIES, INCLUDING WITHOUT LIMITATION, ANY
// WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, QUIET
// ENJOYMENT, OR NON-INFRINGEMENT. See ../license.md for the RPL's specific
// language governing rights and limitations.

//! Minimal CUDA Driver ABI ownership; all allocations and copies execute on CUDA.
//! ABI: https://docs.nvidia.com/cuda/cuda-driver-api/index.html
use anyhow::{Context, Result, bail};
use libloading::Library;
use std::{ffi::c_void, marker::PhantomData, rc::Rc};
type Status = i32;
type Handle = *mut c_void;

pub(crate) struct Cuda {
    pub(crate) identity: serde_json::Value,
    device: i32,
    release: unsafe extern "C" fn(i32) -> Status,
    alloc: unsafe extern "C" fn(*mut u64, usize) -> Status,
    free: unsafe extern "C" fn(u64) -> Status,
    htod: unsafe extern "C" fn(u64, *const c_void, usize) -> Status,
    dtoh: unsafe extern "C" fn(*mut c_void, u64, usize) -> Status,
    stream_create: unsafe extern "C" fn(*mut Handle, u32) -> Status,
    stream_sync: unsafe extern "C" fn(Handle) -> Status,
    stream_destroy: unsafe extern "C" fn(Handle) -> Status,
    // Context and allocations stay on their owning worker thread.
    _thread: PhantomData<Rc<()>>,
    _library: Library,
}
fn check(status: Status, operation: &str) -> Result<()> {
    if status != 0 {
        bail!("CUDA {operation} failed with driver status {status}");
    }
    Ok(())
}
impl Cuda {
    pub(crate) fn open(ordinal: i32, library: &std::path::Path) -> Result<Self> {
        // SAFETY: fixed CUDA Driver library and documented C ABI signatures. The
        // library remains loaded until after every context, stream and buffer drops.
        unsafe {
            let lib = Library::new(library).context("load real NVIDIA CUDA driver libcuda.so.1")?;
            let init: unsafe extern "C" fn(u32) -> Status = *lib.get(b"cuInit\0")?;
            let count: unsafe extern "C" fn(*mut i32) -> Status =
                *lib.get(b"cuDeviceGetCount\0")?;
            let get: unsafe extern "C" fn(*mut i32, i32) -> Status = *lib.get(b"cuDeviceGet\0")?;
            let version: unsafe extern "C" fn(*mut i32) -> Status =
                *lib.get(b"cuDriverGetVersion\0")?;
            let uuid: unsafe extern "C" fn(*mut [u8; 16], i32) -> Status =
                *lib.get(b"cuDeviceGetUuid\0")?;
            let retain: unsafe extern "C" fn(*mut Handle, i32) -> Status =
                *lib.get(b"cuDevicePrimaryCtxRetain\0")?;
            let set: unsafe extern "C" fn(Handle) -> Status = *lib.get(b"cuCtxSetCurrent\0")?;
            let release: unsafe extern "C" fn(i32) -> Status =
                *lib.get(b"cuDevicePrimaryCtxRelease_v2\0")?;
            // Resolve everything before retaining a context, so a missing symbol leaks nothing.
            let mut cuda = Self {
                identity: serde_json::Value::Null,
                device: -1,
                release,
                alloc: *lib.get(b"cuMemAlloc_v2\0")?,
                free: *lib.get(b"cuMemFree_v2\0")?,
                htod: *lib.get(b"cuMemcpyHtoD_v2\0")?,
                dtoh: *lib.get(b"cuMemcpyDtoH_v2\0")?,
                stream_create: *lib.get(b"cuStreamCreate\0")?,
                stream_sync: *lib.get(b"cuStreamSynchronize\0")?,
                stream_destroy: *lib.get(b"cuStreamDestroy_v2\0")?,
                _thread: PhantomData,
                _library: lib,
            };
            check(init(0), "cuInit")?;
            let mut devices = 0;
            check(count(&mut devices), "cuDeviceGetCount")?;
            if ordinal < 0 || ordinal >= devices {
                bail!("CUDA device {ordinal} unavailable; {devices} visible devices");
            }
            let mut device = 0;
            check(get(&mut device, ordinal), "cuDeviceGet")?;
            let mut driver = 0;
            let mut identifier = [0u8; 16];
            check(version(&mut driver), "cuDriverGetVersion")?;
            check(uuid(&mut identifier, device), "cuDeviceGetUuid")?;
            cuda.identity = serde_json::json!({"ordinal":ordinal, "driver_version":driver,
                "uuid":identifier.iter().map(|b| format!("{b:02x}")).collect::<String>()});
            let mut ctx = std::ptr::null_mut();
            check(retain(&mut ctx, device), "cuDevicePrimaryCtxRetain")?;
            cuda.device = device;
            check(set(ctx), "cuCtxSetCurrent")?;
            Ok(cuda)
        }
    }
    pub(crate) fn allocate(&self, count: usize) -> Result<Buffer<'_>> {
        let bytes = count
            .checked_mul(8)
            .context("CUDA allocation size overflow")?;
        let mut pointer = 0;
        // SAFETY: valid output slot and a nonzero size; even zero-count NCCL calls
        // receive a real device pointer, never a Rust heap or synthetic result.
        check(
            unsafe { (self.alloc)(&mut pointer, bytes.max(8)) },
            "cuMemAlloc",
        )?;
        Ok(Buffer {
            cuda: self,
            pointer,
            count,
        })
    }
    pub(crate) fn stream(&self) -> Result<Stream<'_>> {
        let mut raw = std::ptr::null_mut();
        // SAFETY: valid output pointer; CU_STREAM_NON_BLOCKING = 1.
        check(
            unsafe { (self.stream_create)(&mut raw, 1) },
            "cuStreamCreate",
        )?;
        Ok(Stream { cuda: self, raw })
    }
}
impl Drop for Cuda {
    fn drop(&mut self) {
        if self.device >= 0 {
            // SAFETY: paired retain, after all borrowed buffers and streams have dropped.
            unsafe {
                (self.release)(self.device);
            }
        }
    }
}
pub(crate) struct Buffer<'a> {
    cuda: &'a Cuda,
    pointer: u64,
    count: usize,
}
impl Buffer<'_> {
    pub(crate) fn pointer(&self) -> *mut f64 {
        self.pointer as *mut f64
    }
    pub(crate) fn upload(&mut self, values: &[f64]) -> Result<()> {
        if values.len() != self.count {
            bail!("CUDA upload size mismatch");
        }
        if !values.is_empty() {
            // SAFETY: live host slice and owned device allocation of precisely this size.
            check(
                unsafe {
                    (self.cuda.htod)(
                        self.pointer,
                        values.as_ptr().cast(),
                        std::mem::size_of_val(values),
                    )
                },
                "cuMemcpyHtoD",
            )?;
        }
        Ok(())
    }
    /// Caller must finish all NCCL work before reading or dropping the buffer.
    pub(crate) fn download(&self) -> Result<Vec<f64>> {
        let mut values = vec![0.0; self.count];
        if self.count != 0 {
            // SAFETY: destination has count writable f64s and NCCL's stream is complete.
            check(
                unsafe {
                    (self.cuda.dtoh)(values.as_mut_ptr().cast(), self.pointer, self.count * 8)
                },
                "cuMemcpyDtoH",
            )?;
        }
        Ok(values)
    }
}
impl Drop for Buffer<'_> {
    fn drop(&mut self) {
        // SAFETY: allocation belongs to the retained current context. The worker
        // synchronizes/aborts NCCL before these buffers are dropped.
        unsafe {
            (self.cuda.free)(self.pointer);
        }
    }
}
pub(crate) struct Stream<'a> {
    cuda: &'a Cuda,
    raw: Handle,
}
impl Stream<'_> {
    pub(crate) fn nccl(&self) -> nccl::CudaStream {
        // SAFETY: CUDA runtime and driver stream handles share the same ABI.
        unsafe { nccl::CudaStream::from_raw(self.raw.cast()) }
    }
    pub(crate) fn synchronize(&self) -> Result<()> {
        // SAFETY: live stream in its owning thread's current CUDA context.
        check(
            unsafe { (self.cuda.stream_sync)(self.raw) },
            "cuStreamSynchronize",
        )
    }
}
impl Drop for Stream<'_> {
    fn drop(&mut self) {
        // SAFETY: owned stream, after completion or communicator abort.
        unsafe {
            (self.cuda.stream_destroy)(self.raw);
        }
    }
}
