from abc import ABC, abstractmethod
import struct

from values import String, Integer, Float
import tokenizer

from instructions import Instruction

class Macro(ABC):
    def __init__(self, span):
        self.span = span

    @abstractmethod
    def __repr__(self):
        pass

    def __str__(self):
        return repr(self)

    @abstractmethod
    def into_pseudo_values(self):
        pass

    @abstractmethod
    def post_process(self, output):
        pass

class LabelPseudoInstruction(Instruction):
    def __init__(self, variable_name):
        super(LabelPseudoInstruction, self).__init__(bytes([]), [])

        self.variable_name = variable_name

    def compile_to_bytecode(self, output):
        output.variables[self.variable_name] = len(output.output)

    def __str__(self):
        return f"LabelPseudoInstruction(variable_name={self.variable_name})"

    __repr__ = __str__

class Label(Macro):
    def __init__(self, span, name):
        super(Label, self).__init__(span)

        self.name = name

    def __repr__(self):
        return f"Label(name={repr(self.name)})"

    def into_pseudo_values(self):
        return [LabelPseudoInstruction(self.name)]

    def post_process(self, output):
        pass

class Eq(Macro):
    def __init__(self, span, name):
        super(Eq, self).__init__(span)

        self.name = name

        self.equal = Integer(tokenizer.IntegerToken(span, 0))

    def __repr__(self):
        return f"Eq(name={repr(self.name)})"

    def into_pseudo_values(self):
        return [self.equal]

    def post_process(self, output):
        if self.name not in output.variables:
            self.span.print_aa()
            print("Variable not found")
            exit()

        value = output.variables[self.name]

        track_id = self.equal.track_id
        position = output.tracked_ids[track_id]

        output.override_inside(position, struct.pack("Q", value))

class DeclarePseudoInstruction(Instruction):
    def __init__(self, variable_name, value):
        super(DeclarePseudoInstruction, self).__init__(bytes([]), [])

        self.variable_name = variable_name
        self.value = value

    def compile_to_bytecode(self, output):
        output.variables[self.variable_name] = self.value

    def __str__(self):
        return f"DeclarePseudoInstruction(variable_name={self.variable_name}, value={self.value})"

    __repr__ = __str__

class Declare(Macro):
    def __init__(self, span, name, value):
        super(Declare, self).__init__(span)

        self.name = name
        self.value = value

    def __repr__(self):
        return f"Declare(name={repr(self.name)}, value={self.value})"

    def into_pseudo_values(self):
        return [DeclarePseudoInstruction(self.name, self.value)]

    def post_process(self, output):
        pass

class IncPseudoInstruction(Instruction):
    def __init__(self, category, variable):
        super(IncPseudoInstruction, self).__init__(bytes([]), [])

        self.category = category
        self.variable = variable

    def compile_to_bytecode(self, output):
        output.variables[self.variable] = output.variables[self.category]
        output.variables[self.category] += 1

    def __str__(self):
        return f"IncPseudoInstruction(category={self.category}, variable={self.variable})"

    __repr__ = __str__

class Inc(Macro):
    def __init__(self, span, category, variable):
        super(Inc, self).__init__(span)

        self.category = category
        self.variable = variable

    def __repr__(self):
        return f"Inc(category={self.category}, variable={self.variable})"

    def into_pseudo_values(self):
        return [IncPseudoInstruction(self.category, self.variable)]

    def post_process(self, output):
        pass

def construct_macro(macro_token, argument_tokens):
    if len(argument_tokens) == 0:
        full_span = macro_token.span
    else:
        full_span = macro_token.span.combine_nonadjacent(argument_tokens[-1].span)

    if macro_token.macro_name == "label":
        if len(argument_tokens) == 1:
            if isinstance(argument_tokens[0], tokenizer.StringToken):
                return Label(full_span, argument_tokens[0].value)
            else:
                argument_tokens[0].span.print_aa()
                print("@label needs a string")
                exit()
        else:
            if len(argument_tokens) == 0:
                macro_token.span.print_aa()
            else:
                argument_span = argument_tokens[0].span.combine_nonadjacent(argument_tokens[-1].span)
                argument_span.print_aa()
            print("@label needs exactly one argument")
            exit()
    if macro_token.macro_name == "declare":
        if len(argument_tokens) == 2:
            if isinstance(argument_tokens[0], tokenizer.StringToken):
                if isinstance(argument_tokens[1], tokenizer.IntegerToken):
                    return Declare(full_span, argument_tokens[0].value, argument_tokens[1].value)
                else:
                    argument_tokens[1].span.print_aa()
                    print("@declare's second argument is an integer")
                    exit()
            else:
                argument_tokens[0].span.print_aa()
                print("@declare's first argument is a string")
                exit()
        else:
            if len(argument_tokens) == 0:
                macro_token.span.print_aa()
            else:
                argument_span = argument_tokens[0].span.combine_nonadjacent(argument_tokens[-1].span)
                argument_span.print_aa()
            print("@declare needs exactly two argument")
            exit()
    if macro_token.macro_name == "inc":
        if len(argument_tokens) == 2:
            if isinstance(argument_tokens[0], tokenizer.StringToken):
                if isinstance(argument_tokens[0], tokenizer.StringToken):
                    return Inc(full_span, argument_tokens[0].value, argument_tokens[1].value)
                else:
                    argument_tokens[1].span.print_aa()
                    print("@inc's second argument is a string")
                    exit()
            else:
                argument_tokens[0].span.print_aa()
                print("@inc's first argument is a string")
                exit()
        else:
            if len(argument_tokens) == 0:
                macro_token.span.print_aa()
            else:
                argument_span = argument_tokens[0].span.combine_nonadjacent(argument_tokens[-1].span)
                argument_span.print_aa()
            print("@inc needs exactly two argument")
            exit()
    elif macro_token.macro_name == "=":
        if len(argument_tokens) == 1:
            if isinstance(argument_tokens[0], tokenizer.StringToken):
                return Eq(full_span, argument_tokens[0].value)
            else:
                argument_tokens[0].span.print_aa()
                print("@= needs a string")
                exit()
        else:
            if len(argument_tokens) == 0:
                macro_token.span.print_aa()
            else:
                argument_span = argument_tokens[0].span.combine_nonadjacent(argument_tokens[-1].span)
                argument_span.print_aa()
            print("@= needs exactly one argument")
            exit()
    else:
        macro_token.span.print_aa()
        print("Unknown macro", macro_token.macro_name)
        exit()
