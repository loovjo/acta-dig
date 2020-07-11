from tokenizer import Register, String, Integer, Float

class Instruction:
    def __init__(self, mnemonic, bytecode_byte):
        self.mnemonic = mnemonic
        self.bytecode_byte = bytecode_byte
        self.argument_verifiers = []

    def verify(self, message, fn):
        new = Instruction(self.mnemonic)
        new.argument_verifiers = self.argument_verifiers[:]
        new.argument_verifiers.append((message, argument_verifiers))
        return new

INSTRUCTIONS = \
    [ Instruction("set_self_addr", 0x00)
        .verify("needs one argument", lambda args: len(args) == 1))
        .verify("argument one has to be register", lambda args: isinstance(args[0], Register))
    , Instruction("set_float", 0x01)
        .verify("needs two arguments", lambda args: len(args) == 2))
        .verify("argument one has to be register", lambda args: isinstance(args[0], Register))
        .verify("argument two has to be float", lambda args: isinstance(args[1], Float))
    , Instruction("set_integer", 0x02)
        .verify("needs two arguments", lambda args: len(args) == 2))
        .verify("argument one has to be register", lambda args: isinstance(args[0], Register))
        .verify("argument two has to be int", lambda args: isinstance(args[1], Integer))
    , Instruction("set_string", 0x03)
        .verify("needs two arguments", lambda args: len(args) == 2))
        .verify("argument one has to be register", lambda args: isinstance(args[0], Register))
        .verify("argument two has to be string", lambda args: isinstance(args[1], String))
    , Instruction("copy", 0x04)
        .verify("needs two arguments", lambda args: len(args) == 2))
        .verify("argument one has to be register", lambda args: isinstance(args[0], Register))
        .verify("argument two has to be register", lambda args: isinstance(args[1], Register))
    , Instruction("generate_atom", 0x05)
        .verify("needs one argument", lambda args: len(args) == 1))
        .verify("argument one has to be register", lambda args: isinstance(args[0], Register))
    # TODO: More instructions
    ]
