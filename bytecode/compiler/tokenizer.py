from abc import ABC, abstractmethod

from span import Span

class ParseInput:
    def __init__(self, file_cont, cursor=0):
        self.file_cont = file_cont
        self.cursor = cursor

    def peek(self):
        if self.cursor == len(self.file_cont):
            return Span(self.cursor, self.cursor, self.file_cont)
        return Span(self.cursor, self.cursor + 1, self.file_cont)

    def pop(self):
        span = self.peek()
        return (span, ParseInput(self.file_cont, span.end))

class ParseException(Exception):
    def __init__(self, reason, span):
        super(ParseException, self).__init__()
        self.reason = reason
        self.span = span

    def __str__(self):
        return f"ParseException(reason={repr(self.reason)}, span={self.span})"

    __repr__ = __str__

class Token(ABC):
    def __init__(self, span):
        self.span = span

    @abstractmethod
    def parse_one(inp):
        pass

    @abstractmethod
    def __repr__(self):
        pass

    def __str__(self):
        return repr(self)

def map_res(res_wrapper, parse_fn):
    def parse_one(inp):
        data, inp = parse_fn(inp)

        return res_wrapper(data), inp

    return parse_one

def change_reason(reason, parse_fn):
    def parse_one(inp):
        try:
            return parse_fn(inp)
        except ParseException as e:
            e.reason = reason
            raise e

    return parse_one

def take_while(fn, err_reason):
    fn_ = lambda x: x != "" and fn(x)
    def parse_one(inp):
        if fn_(inp.peek().get()):
            span, inp = inp.pop()
            while fn_(inp.peek().get()):
                next, inp = inp.pop()
                span = span.combine(next)

            return span, inp
        else:
            raise ParseException(err_reason, inp.peek())

    return parse_one

def take_while0(fn):
    fn_ = lambda x: x != "" and fn(x)
    def parse_one(inp):
        if fn_(inp.peek().get()):
            span, inp = inp.pop()
            while fn_(inp.peek().get()):
                next, inp = inp.pop()
                span = span.combine(next)

            return span, inp
        else:
            return Span(inp.cursor, inp.cursor, inp.file_cont), inp

    return parse_one

parse_spaces = take_while0(lambda x: x in " \n")

NONBREAKING_CHARS = "abcdefghijklmnopqrstuvwxyz_0123456789"
def parse_spaces1(inp):
    if inp.peek().get() not in NONBREAKING_CHARS:
        return Span(inp.cursor, inp.cursor, inp.file_cont), inp

    return take_while(lambda x: x in " \n", "Expected space")(inp)

def spaced(parse_fn):
    def parse_one(inp):
        _, inp = parse_spaces(inp)
        data, inp = parse_fn(inp)
        if inp.cursor != len(inp.file_cont):
            _, inp = parse_spaces1(inp)

        return data, inp

    return parse_one

def spaced_light(parse_fn):
    def parse_one(inp):
        _, inp = parse_spaces(inp)
        data, inp = parse_fn(inp)
        _, inp = parse_spaces(inp)

        return data, inp

    return parse_one

parse_int = map_res(
    lambda span: (span, int(span.get())),
    take_while(lambda x: x in "0123456789", "Expected integer")
)

def find_exact(st):
    def parse_one(inp):
        if inp.file_cont[inp.cursor:inp.cursor + len(st)] == st:
            span, inp = inp.pop()
            for _ in range(len(st) - 1):
                span, inp = span.combine(inp.pop())
            return span, inp
        raise ParseException(f"Expected {st}", inp.peek())

    return parse_one


class AssemblyInstructionToken(Token):
    def __init__(self, span, inst):
        super(AssemblyInstructionToken, self).__init__(span)
        self.inst = inst

    PARSE_TOKEN = take_while(lambda ch: ch in "abcdefghijklmnopqrstuvwxyz_", "Expected token")
    @spaced
    def parse_one(inp):
        span, inp = AssemblyInstructionToken.PARSE_TOKEN(inp)

        return AssemblyInstructionToken(span, span.get()), inp

    def __repr__(self):
        return f"AssemblyInstructionToken(inst={repr(self.inst)})"

class MacroToken(Token):
    def __init__(self, span, macro_name):
        super(MacroToken, self).__init__(span)
        self.macro_name = macro_name

    PARSE_MACRO = take_while(lambda ch: ch in "abcdefghijklmnopqrstuvwxyz_=", "Expected token")
    def parse_one(inp):
        _, inp = parse_spaces(inp)
        span, inp = find_exact("@")(inp)
        macro_name, inp = MacroToken.PARSE_MACRO(inp)
        _, inp = find_exact("(")(inp)
        _, inp = parse_spaces(inp)

        return MacroToken(span.combine(macro_name), macro_name.get()), inp

    def __repr__(self):
        return f"MacroToken(inst={repr(self.macro_name)})"

class MacroEndToken(Token):
    def __init__(self, span):
        super(MacroEndToken, self).__init__(span)

    def parse_one(inp):
        _, inp = parse_spaces(inp)
        span, inp = find_exact(")")(inp)
        _, inp = parse_spaces(inp)

        return MacroEndToken(span), inp

    def __repr__(self):
        return f"MacroToken()"

class RegisterToken(Token):
    def __init__(self, span, reg_idx):
        super(RegisterToken, self).__init__(span)
        self.reg_idx = reg_idx

    LETTER_R = find_exact("r")
    @spaced
    def parse_one(inp):
        span, inp = RegisterToken.LETTER_R(inp)
        (ispan, reg_idx), inp = parse_int(inp)

        if 0 <= reg_idx < 256:
            return RegisterToken(span.combine(ispan), reg_idx), inp
        else:
            raise ParseException("Register outside range 0-255", ispan)

    def __repr__(self):
        return f"RegisterToken(reg_idx={self.reg_idx})"

class CommentToken(Token):
    CommentTokenStart = find_exact(";")
    UntilEOL = take_while(lambda x: x != "\n", "Expected newline")

    @spaced
    def parse_one(inp):
        comment_syntax, inp = CommentToken.CommentTokenStart(inp)
        comment, inp = CommentToken.UntilEOL(inp)

        return CommentToken(comment_syntax.combine(comment)), inp

    def __repr__(self):
        return f"CommentToken"

class Char(Token):
    def __init__(self, span, value):
        super(Char, self).__init__(span)
        self.value = value

    ESCAPE_TABLE = {
        "n": "\n",
        "t": "\t",
        "\\": "\\",
        "'": "'",
        # TODO: Add more?
    }
    def parse_one(inp):
        ch, inp = inp.pop()
        if ch.get() == '\\':
            next, inp = inp.pop()
            if next.get() in Char.ESCAPE_TABLE:
                return Char(ch.combine(next), Char.ESCAPE_TABLE[next.get()]), inp
            else:
                lst = ["\\" + ch for ch in Char.ESCAPE_TABLE.keys()]
                raise ParseException(f"Expected any of {', '.join(lst)}", ch.combine(next))
        else:
            return Char(ch, ch.get()), inp

    def __repr__(self):
        return f"Char(value={repr(self.value)})"

class StringToken(Token):
    def __init__(self, span, value):
        super(StringToken, self).__init__(span)
        self.value = value

    START_END = find_exact("'")
    @spaced
    def parse_one(inp):
        span, inp = StringToken.START_END(inp)

        value = ''
        while True:
            try:
                end, inp = StringToken.START_END(inp)
                span = span.combine(end)
                break
            except ParseException as _:
                ch, inp = Char.parse_one(inp)
                span = span.combine(ch.span)
                value += ch.value

        return StringToken(span, value), inp

    def __repr__(self):
        return f"StringToken(value={repr(self.value)})"

class IntegerToken(Token):
    def __init__(self, span, value):
        super(IntegerToken, self).__init__(span)
        self.value = value

    @spaced
    def parse_one(inp):
        (span, number), inp = parse_int(inp)

        return IntegerToken(span, number), inp

    def __repr__(self):
        return f"IntegerToken(value={self.value})"


class FloatToken(Token):
    def __init__(self, span, value):
        super(FloatToken, self).__init__(span)
        self.value = value

    @spaced
    def parse_one(inp):
        # TODO: This currently breaks on things like 1e5, because it assumes a prefix of a valid
        # float is a valid float. '1e' is not a valid float
        longest_flength = 0
        value = None
        for flength in range(1, len(inp.file_cont) - inp.cursor):
            try:
                st = inp.file_cont[inp.cursor:inp.cursor+flength]
                if " " in st or "\n" in st:
                    raise ValueError()
                value = float(st)
                if "." in st:
                    longest_flength = flength
            except ValueError as _:
                break

        if longest_flength == 0:
            raise ParseException("Expected float", Span(inp.cursor, inp.cursor + 1, inp.file_cont))

        span = Span(inp.cursor, inp.cursor + longest_flength, inp.file_cont)
        inp.cursor += longest_flength

        return FloatToken(span, value), inp

    def __repr__(self):
        return f"FloatToken(value={self.value})"

priority_order = [
    MacroToken, MacroEndToken, RegisterToken, FloatToken, IntegerToken, StringToken, AssemblyInstructionToken, CommentToken,
]

def parse_one(inp):
    furthest_exception = None
    for thing in priority_order:
        try:
            return thing.parse_one(inp)
        except ParseException as x:
            if furthest_exception is None or furthest_exception.span.end < x.span.end:
                furthest_exception = x

    raise furthest_exception

def parse_all(inp):
    _, inp = parse_spaces(inp)
    if inp.cursor == len(inp.file_cont):
        return []

    thing, inp  = parse_one(inp)
    rest = parse_all(inp)
    return [thing] + rest

if __name__ == "__main__":
    inp_text = open("../test_1.dig").read()
    inp_pi = ParseInput(inp_text)
    try:
        for thing in parse_all(inp_pi):
            print(thing)
            thing.span.print_aa()
            print()
    except ParseException as p:
        print("Error:", p.reason)
        p.span.print_aa()
        raise p
