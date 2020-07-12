from abc import ABC, abstractmethod

import random

import struct

import tokenizer

class CompileOutput:
    def __init__(self):
        self.output = b''

        self.tracked_ids = {}
        self.variables = {}

    def write_bytes(self, b):
        addr = len(self.output)
        self.output = self.output + b

        return addr

    def __str__(self):
        return f"CompileOutput(output={self.output})"

    def __repr__(self):
        return f"CompileOutput(output={self.output}, tracked_ids={self.tracked_ids}, variables={self.variables})"

class Value(ABC):
    def __init__(self, inner):
        self.inner = inner
        self.track_id = random.getrandbits(64)

    @abstractmethod
    def __repr__(self):
        pass

    def __str__(self):
        return repr(self)

    def compile_to_bytecode(self, output):
        addr = output.write_bytes(self.get_bytecode())
        output.tracked_ids[self.track_id] = addr

    @abstractmethod
    def get_bytecode(self):
        pass

class String(Value):
    def get_bytecode(self):
        enc_bytes = self.inner.value.encode("utf-8")
        return struct.pack("Q", len(enc_bytes)) + enc_bytes

    def __repr__(self):
        return f"String(inner={repr(self.inner)})"

class Integer(Value):
    def get_bytecode(self):
        return struct.pack("Q", self.inner.value)

    def __repr__(self):
        return f"Integer(inner={repr(self.inner)})"

class Float(Value):
    def get_bytecode(self):
        return struct.pack("d", self.inner.value)

    def __repr__(self):
        return f"Float(inner={repr(self.inner)})"

class Register(Value):
    def get_bytecode(self):
        return bytes([self.inner.reg_idx])

    def __repr__(self):
        return f"Register(inner={repr(self.inner)})"

def upgrade(value):
    if isinstance(value, tokenizer.StringToken):
        return String(value)

    if isinstance(value, tokenizer.IntegerToken):
        return Integer(value)

    if isinstance(value, tokenizer.FloatToken):
        return Float(value)

    if isinstance(value, tokenizer.RegisterToken):
        return Register(value)

    return value
