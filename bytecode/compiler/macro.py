from abc import ABC, abstractmethod

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

# TODO: Category counting

class Eq(Macro):
    def __init__(self, span, name):
        super(Eq, self).__init__(span)

        self.name = name

        self.equal = Integer(tokenizer.IntegerToken(None, 0))

    def __repr__(self):
        return f"Eq(name={repr(self.name)})"

    def into_pseudo_values(self):
        return [self.equal]

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
    elif macro_token.macro_name == "=":
        if len(argument_tokens) == 1:
            if isinstance(argument_tokens[0], tokenizer.StringToken):
                return Eq(full_span, argument_tokens[0].value)
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
    else:
        macro_token.span.print_aa()
        print("Unknown macro", macro_token.macro_name)
        exit()
