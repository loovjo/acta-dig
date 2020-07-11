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

class Token(ABC):
    def __init__(self, span):
        self.span = span

    @abstractmethod
    def parse_one(inp):
        pass

def take_while(fn):
    fn_ = lambda x: x != "" and fn(x)
    class TakeWhile(Token):
        def parse_one(inp):
            if fn_(inp.peek().get()):
                span, inp = inp.pop()
                while fn_(inp.peek().get()):
                    next, inp = inp.pop()
                    span = span.combine(next)

                return TakeWhile(span), inp
            else:
                raise ParseException("Not found", inp.peek())

    return TakeWhile

def take_while0(fn):
    fn_ = lambda x: x != "" and fn(x)
    class TakeWhile0(Token):
        def parse_one(inp):
            if fn_(inp.peek().get()):
                span, inp = inp.pop()
                while fn_(inp.peek().get()):
                    next, inp = inp.pop()
                    span = span.combine(next)

                return TakeWhile0(span), inp
            else:
                return TakeWhile0(Span(inp.cursor, inp.cursor, inp.file_cont)), inp

    return TakeWhile0

Spaces = take_while0(lambda x: x in " \n")

def spaced(Token):
    class Spaced(Token.__class__):
        def parse_one(inp):
            _, inp = Spaces.parse_one(inp)
            token, inp = Token.parse_one(inp)
            _, inp = Spaces.parse_one(inp)

            return token, inp

    return Spaced

def find_exact(st):
    class FindExact(Token):
        def parse_one(inp):
            if inp.file_cont[inp.cursor:inp.cursor + len(st)] == st:
                span, inp = inp.pop()
                for _ in range(len(st) - 1):
                    span, inp = span.combine(inp.pop())
                return FindExact(span), inp
            raise ParseException(f"Expected {st}", inp.peek())

    return FindExact

AssemblyInstructionToken = spaced(take_while(lambda ch: ch in "abcdefghijklmnopqrstuvwxyz_"))

class Comment(Token):
    CommentStart = find_exact(";")
    UntilEOL = take_while(lambda x: x != "\n")

    def parse_one(inp):
        comment_syntax, inp = Comment.CommentStart.parse_one(inp)
        comment, inp = Comment.UntilEOL.parse_one(inp)

        return Comment(comment_syntax.span.combine(comment.span)), inp

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
    def parse_one(inp):
        start, inp = String.START_END.parse_one(inp)
        span = start.span

        value = ''
        while True:
            try:
                end, inp = String.START_END.parse_one(inp)
                span = span.combine(end.span)
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
        number, inp = Integer.NUMBER.parse_one(inp)

        return Integer(number.span, int(number.span.get())), inp

class Float(Token):
    def __init__(self, span, value):
        super(Float, self).__init__(span)
        self.value = value

    def parse_one(inp):
        # TODO: This currently breaks on things like 1e5, because it assumes a prefix of a valid
        # float is a valid float. '1e' is not a valid float
        longest_flength = 0
        for flength in range(1, len(inp.file_cont) - inp.cursor):
            try:
                st = inp.file_cont[inp.cursor:inp.cursor+flength]
                if " " in st:
                    raise ValueError()
                value = float(st)
                longest_flength = flength
            except ValueError as _:
                if inp.file_cont[inp.cursor+flength-1] == "e":
                    continue
                break

        span = Span(inp.cursor, inp.cursor + longest_flength, inp.file_cont)
        inp.cursor += longest_flength

        return Float(span, value), inp

inp_text = r"123 yo"
inp_pi = ParseInput(inp_text)
try:
    inst, inp_pi = Float.parse_one(inp_pi)
    inst.span.print_aa()
    print(inst.value)

    inst, inp_pi = AssemblyInstructionToken.parse_one(inp_pi)
    inst.span.print_aa()
except ParseException as p:
    print("Error:", p.reason)
    p.span.print_aa()
    raise p
