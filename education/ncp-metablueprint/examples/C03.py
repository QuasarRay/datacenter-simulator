import hashlib
from unpythonic import pipe1
from ncp_lab.macros import macros, require
payload = b"teaching artifact, not a GPU binary"
identity = pipe1(payload, hashlib.sha256, lambda h: h.hexdigest())
manifest = {"digest": identity, "os": "cachyos", "execution": "native-incus"}
require[manifest["digest"] == hashlib.sha256(payload).hexdigest()]
require[manifest["execution"] == "native-incus"]
