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

inp_text = "; hello world \n yo yo"
inp_pi = ParseInput(inp_text)
try:
    inst, inp_pi = Comment.parse_one(inp_pi)
    inst.span.print_aa()

    inst, inp_pi = AssemblyInstructionToken.parse_one(inp_pi)
    inst.span.print_aa()
except ParseException as p:
    print("Error:", p.reason)
    p.span.print_aa()
    raise p
