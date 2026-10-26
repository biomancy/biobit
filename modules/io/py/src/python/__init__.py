from biobit.rs import io as _native

from biobit import _masquerade_native_imports

from . import protocols as _protocols

# Protocols are Python typing contracts; all stream implementations are native.
# Retain their public module before replacing this package in the import registry.
_native.ReadRecord = _protocols.ReadRecord
_native.protocols = _protocols
_native.__all__ = ["ReadRecord", *_native.__all__]
_masquerade_native_imports(_native, __name__)
