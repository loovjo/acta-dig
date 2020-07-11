from abc import ABC, abstractmethod

from tokenizer import Macro as MacroToken, AssemblyInstructionToken, Comment, ParseInput, parse_all

class SyntaxElement(ABC):
    def __init__(self, span):
        self.span = span

    @abstractmethod
    def compile_and_info(self, compile_info):
        pass

    @abstractmethod
    def __repr__(self):
        pass

    def __str__(self):
        return __repr__(self)

class Instruction(SyntaxElement):
    def __init__(self, span, mnemonic, argument_list):
        super(Instruction, self).__init__(span)

        self.mnemonic = mnemonic
        self.argument_list = argument_list

    def compile_and_info(self, compile_info):
        raise NotImplementedError()

    def __repr__(self):
        return f"Instruction(mnemonic={repr(self.mnemonic)}, argument_list={self.argument_list})"

class Macro(SyntaxElement):
    def __init__(self, span, macro_name, argument_list):
        super(Macro, self).__init__(span)

        self.macro_name = macro_name
        self.argument_list = argument_list

    def compile_and_info(self, compile_info):
        raise NotImplementedError()

    def __repr__(self):
        return f"Macro(macro_name={repr(self.macro_name)}, argument_list={self.argument_list})"

def preproc_tokens(token_list):
    return [token for token in token_list if not isinstance(token, Comment)]

def parse_tokens(token_list):
    if len(token_list) == 0:
        return []

    first_token = token_list[0]
    if isinstance(first_token, AssemblyInstructionToken):
        for argument_count in range(len(token_list)):
            rest = parse_tokens(token_list[1 + argument_count:])
            if rest != None:
                arguments = token_list[1:1+argument_count]
                break

        if len(arguments) > 0:
            span = first_token.span.combine_nonadjacent(arguments[-1].span)
        else:
            span = first_token.span

        return [Instruction(span, first_token.inst, arguments)] + rest

    if isinstance(first_token, MacroToken):
        for argument_count in range(len(token_list)):
            rest = parse_tokens(token_list[1 + argument_count:])
            if rest != None:
                arguments = token_list[1:1+argument_count]
                break

        if len(arguments) > 0:
            span = first_token.span.combine_nonadjacent(arguments[-1].span)
        else:
            span = first_token.span

        return [Macro(span, first_token.macro_name, arguments)] + rest


if __name__ == "__main__":
    inp_text = open("../test.dig").read()
    inp_pi = ParseInput(inp_text)
    tokens = parse_all(inp_pi)
    parsed = parse_tokens(tokens)

    print("\n".join(map(repr, parsed)))
