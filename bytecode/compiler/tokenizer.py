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

def take_while(fn):
    fn_ = lambda x: x != "" and fn(x)
    def parse_one(inp):
        if fn_(inp.peek().get()):
            span, inp = inp.pop()
            while fn_(inp.peek().get()):
                next, inp = inp.pop()
                span = span.combine(next)

            return span, inp
        else:
            raise ParseException("Not found", inp.peek())

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

def spaced(parse_fn):
    def parse_one(inp):
        _, inp = parse_spaces(inp)
        data, inp = parse_fn(inp)
        _, inp = parse_spaces(inp)

        return data, inp

    return parse_one

def find_exact(st):
    def parse_one(inp):
        if inp.file_cont[inp.cursor:inp.cursor + len(st)] == st:
            span, inp = inp.pop()
            for _ in range(len(st) - 1):
                span, inp = span.combine(inp.pop())
            return span, inp
        raise ParseException(f"Expected {st}", inp.peek())

    return parse_one

def map_res(res_wrapper, parse_fn):
    def parse_one(inp):
        data, inp = parse_fn(inp)

        return res_wrapper(data), inp

    return parse_one

class AssemblyInstructionToken(Token):
    def __init__(self, span, inst):
        super(AssemblyInstructionToken, self).__init__(span)
        self.inst = inst

    @spaced
    def parse_one(inp):
        span, inp = spaced(take_while(lambda ch: ch in "abcdefghijklmnopqrstuvwxyz_"))(inp)

        return AssemblyInstructionToken(span, span.get()), inp

class Comment(Token):
    CommentStart = find_exact(";")
    UntilEOL = take_while(lambda x: x != "\n")

    def parse_one(inp):
        comment_syntax, inp = Comment.CommentStart(inp)
        comment, inp = Comment.UntilEOL(inp)

        return Comment(comment_syntax.combine(comment)), inp

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

class String(Token):
    def __init__(self, span, value):
        super(String, self).__init__(span)
        self.value = value

    START_END = find_exact("'")
    @spaced
    def parse_one(inp):
        span, inp = String.START_END(inp)

        value = ''
        while True:
            try:
                end, inp = String.START_END(inp)
                span = span.combine(end)
                break
            except ParseException as _:
                ch, inp = Char.parse_one(inp)
                span = span.combine(ch.span)
                value += ch.value

        return String(span, value), inp

class Integer(Token):
    def __init__(self, span, value):
        super(Integer, self).__init__(span)
        self.value = value

    NUMBER = take_while(lambda x: x in "0123456789")
    def parse_one(inp):
        number, inp = Integer.NUMBER(inp)

        return Integer(number, int(number.get())), inp

class Float(Token):
    def __init__(self, span, value):
        super(Float, self).__init__(span)
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
                longest_flength = flength
            except ValueError as _:
                break

        if value is None:
            raise ParseException("Expected float", Span(inp.cursor, inp.cursor + 1, inp.file_cont))

        span = Span(inp.cursor, inp.cursor + longest_flength, inp.file_cont)
        inp.cursor += longest_flength

        return Float(span, value), inp

inp_text = r"""
7.5
123
'hello\'123\n'
; hello
yo_yo
"""
inp_pi = ParseInput(inp_text)
try:
    inst, inp_pi = Float.parse_one(inp_pi)
    inst.span.print_aa()
    print(inst.value)

    inst, inp_pi = Integer.parse_one(inp_pi)
    inst.span.print_aa()
    print(inst.value)

    inst, inp_pi = String.parse_one(inp_pi)
    inst.span.print_aa()
    print(inst.value)

    inst, inp_pi = Comment.parse_one(inp_pi)
    inst.span.print_aa()

    inst, inp_pi = AssemblyInstructionToken.parse_one(inp_pi)
    inst.span.print_aa()
except ParseException as p:
    print("Error:", p.reason)
    p.span.print_aa()
    raise p
