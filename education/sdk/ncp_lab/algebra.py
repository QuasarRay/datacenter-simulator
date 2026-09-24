"""Pure planning functions, not ASIC, MIG, scheduler or physical validation."""
from dataclasses import dataclass, replace
from math import ceil
from unpythonic import pipe1

def positive(*values):
    if any(type(x) is not int or x<=0 for x in values):
        raise ValueError('positive integer quantities required')

@dataclass(frozen=True)
class Envelope:
    gpus: int
    watts: int
    rack_watts: int
    cooling_watts: int
    admitted: bool = False

def commissioning(value):
    """Produce a new value; do not mutate a partially validated allocation."""
    def quantities(v):
        positive(v.gpus,v.watts,v.rack_watts,v.cooling_watts)
        return v
    def envelope(v):
        if v.watts>min(v.rack_watts,v.cooling_watts):
            raise ValueError('power or cooling envelope exceeded')
        return v
    return pipe1(value,quantities,envelope,lambda v:replace(v,admitted=True))

@dataclass(frozen=True)
class RouteBudget:
    bits_per_second: int
    one_way_ns: int

    def optimistic_ns(self,byte_count):
        positive(self.bits_per_second,byte_count)
        if type(self.one_way_ns) is not int or self.one_way_ns<0:
            raise ValueError('latency must be nonnegative integer nanoseconds')
        return self.one_way_ns+(byte_count*8*1_000_000_000+self.bits_per_second-1)//self.bits_per_second

def headroom_bytes(bits_per_second,stop_ns,in_flight_frame_bytes):
    """Simplified lower bound; ASIC pipeline/rounding/shared-buffer rules are external."""
    positive(bits_per_second,stop_ns,in_flight_frame_bytes)
    return (bits_per_second*stop_ns+8_000_000_000-1)//8_000_000_000+in_flight_frame_bytes

def pkey_allows(a,b):
    """Same nonzero partition base and at least one full member; not an SM emulator."""
    if any(type(x) is not int or not 0<=x<=65535 for x in (a,b)):
        raise ValueError('PKey must be u16')
    return (a&0x7fff)!=0 and (a&0x7fff)==(b&0x7fff) and bool((a|b)&0x8000)

def mig_fits(layout,allowed_layouts):
    """Membership in a trainer-supplied GPU-model-specific placement catalogue.
    Sum of slices is deliberately insufficient; order/placement must be supported.
    This does not change MIG configuration or infer cross-device compatibility.
    """
    return tuple(layout) in {tuple(x) for x in allowed_layouts}
