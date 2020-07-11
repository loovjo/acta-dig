from abc import ABC, abstractmethod

from tokenizer import MacroToken, AssemblyInstructionToken, CommentToken, ParseInput, parse_all

from macro import construct_macro, Macro

import values

def preproc_tokens(token_list):
    return [token for token in token_list if not isinstance(token, CommentToken)]

def group_macros(token_list):
    token_list.append(MacroToken(None, "dummy"))

    result = []

    macro_processing = None # (macro_token, [arguments])

    for token in token_list:
        if macro_processing is not None:
            if isinstance(token, MacroToken) or isinstance(token, AssemblyInstructionToken):
                result.append(construct_macro(*macro_processing))

                macro_processing = None
            else:
                macro_processing[1].append(token)

        if macro_processing is None: # Outside macro
            if isinstance(token, MacroToken):
                macro_processing = (token, [])
            else:
                result.append(token)

    return result

def upgrade_values(element_list):
    return [values.upgrade(element) for element in element_list]

def pseudo_expand_macros(element_list):
    macros = []
    result = []

    for thing in element_list:
        if isinstance(thing, Macro):
            result.extend(thing.into_pseudo_values())
            macros.append(thing)
        else:
            result.append(thing)

    return result, macros

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

        return [Macro(span, first_token.macro_name, arguments)] + rest


if __name__ == "__main__":
    inp_text = open("../test_1.dig").read()
    inp_pi = ParseInput(inp_text)
    tokens = parse_all(inp_pi)
    tokens = preproc_tokens(tokens)
    parsed = group_macros(tokens)
    parsed = upgrade_values(parsed)

    parsed, macros = pseudo_expand_macros(parsed)

    print("\n".join(map(repr, parsed)))
    print("--")
    print("\n".join(map(repr, macros)))
