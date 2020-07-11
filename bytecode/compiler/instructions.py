from values import Register, String, Integer, Float

DISPLAY_ALL = None

class InstructionPattern:
    def __init__(self, mnemonic, bytecode):
        self.mnemonic = mnemonic
        self.bytecode = bytecode
        self.argument_verifiers = []

    def verify(self, message, display, fn):
        new = InstructionPattern(self.mnemonic, self.bytecode)
        new.argument_verifiers = self.argument_verifiers[:]
        new.argument_verifiers.append((message, display, fn))
        return new

class Instruction:
    def __init__(self, bytecode, arguments):
        self.bytecode = bytecode
        self.arguments = arguments

    def __str__(self):
        return f"Instruction(bytecode={self.bytecode}, arguments={self.arguments})"

    __repr__ = __str__

INSTRUCTION_PATTERNS = \
    [ InstructionPattern("set_self_addr", bytes([0x00]))
        .verify("needs one argument", DISPLAY_ALL, lambda args: len(args) == 1)
        .verify("argument one has to be register", 0, lambda args: isinstance(args[0], Register))
    , InstructionPattern("set_float", bytes([0x01]))
        .verify("needs two arguments", DISPLAY_ALL, lambda args: len(args) == 2)
        .verify("argument one has to be register", 0, lambda args: isinstance(args[0], Register))
        .verify("argument two has to be float", 1, lambda args: isinstance(args[1], Float))
    , InstructionPattern("set_integer", bytes([0x02]))
        .verify("needs two arguments", DISPLAY_ALL, lambda args: len(args) == 2)
        .verify("argument one has to be register", 0, lambda args: isinstance(args[0], Register))
        .verify("argument two has to be int", 1, lambda args: isinstance(args[1], Integer))
    , InstructionPattern("set_string", bytes([0x03]))
        .verify("needs two arguments", DISPLAY_ALL, lambda args: len(args) == 2)
        .verify("argument one has to be register", 0, lambda args: isinstance(args[0], Register))
        .verify("argument two has to be string", 1, lambda args: isinstance(args[1], String))
    , InstructionPattern("copy", bytes([0x04]))
        .verify("needs two arguments", DISPLAY_ALL, lambda args: len(args) == 2)
        .verify("argument one has to be register", 0, lambda args: isinstance(args[0], Register))
        .verify("argument two has to be register", 1, lambda args: isinstance(args[1], Register))
    , InstructionPattern("generate_atom", bytes([0x05]))
        .verify("needs one argument", DISPLAY_ALL, lambda args: len(args) == 1)
        .verify("argument one has to be register", 0, lambda args: isinstance(args[0], Register))
    , InstructionPattern("integer_to_atom", bytes([0x05]))
        .verify("needs one argument", DISPLAY_ALL, lambda args: len(args) == 1)
        .verify("argument one has to be register", 0, lambda args: isinstance(args[0], Register))
    , InstructionPattern("dw", bytes([]))
        .verify("needs one argument", DISPLAY_ALL, lambda args: len(args) == 1)
        .verify("argument one has to be int", 0, lambda args: isinstance(args[0], Integer))
    # TODO: More instructions
    ]

def construct_instruction(assembly_instruction_token, arguments):
    for inst in INSTRUCTION_PATTERNS:
        if inst.mnemonic == assembly_instruction_token.inst:
            for (err_msg, displayer, verifier) in inst.argument_verifiers:
                if not verifier(arguments):
                    span = assembly_instruction_token.span
                    if displayer == DISPLAY_ALL and len(arguments) > 0:
                        span = arguments[0].span.combine_nonadjacent(arguments[-1].span)
                    elif displayer < len(arguments):
                        span = arguments[displayer].inner.span
                    span.print_aa()
                    print(err_msg)
                    exit()
            return Instruction(inst.bytecode, arguments)

    assembly_instruction_token.span.print_aa()
    print("No such instruction!")
    # exit()
