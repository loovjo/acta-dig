from abc import ABC, abstractmethod

from tokenizer import MacroToken, MacroEndToken, AssemblyInstructionToken, CommentToken, ParseInput, parse_all

from macro import construct_macro, Macro

import values

from instructions import construct_instruction, Instruction

def preproc_tokens(token_list):
    return [token for token in token_list if not isinstance(token, CommentToken)]

def group_macros(token_list):
    token_list.append(MacroToken(None, "dummy"))

    result = []

    macro_processing = None # (macro_token, [arguments])

    for token in token_list:
        if macro_processing is not None:
            if isinstance(token, MacroEndToken):
                result.append(construct_macro(*macro_processing))

                macro_processing = None
            else:
                macro_processing[1].append(token)

        elif macro_processing is None: # Outside macro
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

def parse_instructions(element_list):
    element_list.append(AssemblyInstructionToken(None, "dummy"))

    result = []
    parsing_instruction = None
    for thing in element_list:
        if isinstance(thing, AssemblyInstructionToken) or isinstance(thing, Instruction):
            if parsing_instruction is not None:
                # Write previous
                result.append(construct_instruction(*parsing_instruction))
                parsing_instruction = None
            if isinstance(thing, AssemblyInstructionToken):
                parsing_instruction = (thing, [])
            else:
                parsing_instruction = None
                result.append(thing)
        else:
            if parsing_instruction is not None:
                parsing_instruction[1].append(thing)
            else:
                print(thing)
                print("Argument to no function")
                exit()

    return result

def compile_to_bytecode(instructions):
    output = values.CompileOutput()
    for inst in instructions:
        inst.compile_to_bytecode(output)

    return output

def postproc_macro(output, macros):
    for macro in macros:
        macro.post_process(output)

if __name__ == "__main__":
    inp_text = open("../test_1.dig").read()
    inp_pi = ParseInput(inp_text)
    tokens = parse_all(inp_pi)
    tokens = preproc_tokens(tokens)
    parsed = group_macros(tokens)
    parsed = upgrade_values(parsed)

    parsed, macros = pseudo_expand_macros(parsed)
    print("\n".join(map(repr, macros)))

    parsed = parse_instructions(parsed)
    print("\n".join(map(repr, parsed)))
    output = compile_to_bytecode(parsed)

    postproc_macro(output, macros)

    print(repr(output))

    with open("/tmp/out.act", "wb") as out:
        out.write(output.output)
