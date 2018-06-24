import lexer

def adapt_strip_newlines(tstream):
    return (t for t in tstream if t.ttype != "newline")

def return_to_iter(item, iterator):
    yield item
    for x in iterator: yield x

def ensure_match_token(token, ttype, tval):
    if token.ttype == ttype and token.tval == tval:
        return
    raise RuntimeError("Token `{}` did not match expected {} `{}` at {}"
        .format(token.tval, ttype, tval, token.lineinfo))

def matching_unwrap_token(token, ttype):
    if token.ttype == ttype:
        return token.tval
    raise RuntimeError("Token `{}` is not a valid {} at {}"
        .format(token.tval, ttype, token.lineinfo))

def error_no_match(token):
    raise RuntimeError("Token `{}` of invalid type `{}` at {}"
        .format(token.tval, token.ttype, token.lineinfo))
    

class Decoder:
    @staticmethod
    def parse(tstream_full):
        tstream = adapt_strip_newlines(tstream_full)
        ensure_match_token(next(tstream), "keyword", "decoder")

        decoder = Decoder()
        decoder.ty = matching_unwrap_token(next(tstream), "label")
        decoder.name = matching_unwrap_token(next(tstream), "label")
        ensure_match_token(next(tstream), "bracket", "{")

        decoder.categories = []
        peeked_next = next(tstream)
        while peeked_next.tval != "}":
            rebuilt_tstream_full = return_to_iter(peeked_next, tstream_full)
            decoder.categories.append(Category.parse(rebuilt_tstream_full))
            peeked_next = next(tstream)
        
        return decoder


class Category:
    @staticmethod
    def parse(tstream_full):
        tstream = adapt_strip_newlines(tstream_full)
        ensure_match_token(next(tstream), "keyword", "category")

        category = Category()
        category.definitions = []
        while True:
            category.definitions.append(Definition.parse(tstream_full))
            next_token = next(tstream)
            if next_token.tval != "or":
                break
            ensure_match_token(next_token, "keyword", "or")

        ensure_match_token(next_token, "bracket", "{")

        category.instructions = []
        peeked_next = next(tstream)
        while peeked_next.tval != "}":
            rebuilt_tstream_full = return_to_iter(peeked_next, tstream_full)
            category.instructions.append(Instruction.parse(rebuilt_tstream_full))
            peeked_next = next(tstream)

        return category


class Instruction:
    @staticmethod
    def parse(tstream_full):
        tstream = adapt_strip_newlines(tstream_full)

        inst = Instruction()
        inst.name = matching_unwrap_token(next(tstream), "label")
        ensure_match_token(next(tstream), "separator", "=")
        inst.defn = Definition.parse(tstream)
        ensure_match_token(next(tstream_full), "newline", "")
        return inst


class Definition:
    @staticmethod
    def parse(tstream_full):
        tstream = adapt_strip_newlines(tstream_full)
        ensure_match_token(next(tstream), "bracket", "[")

        definition = Definition()
        definition.bitgroups = []
        while True:
            definition.bitgroups.append(BitGroup.parse(tstream_full))
            next_token = next(tstream)
            if next_token.tval != ";":
                break
            ensure_match_token(next_token, "separator", ";")
        
        ensure_match_token(next_token, "bracket", "]")
        return definition


class BitGroup:
    class Labeled:
        def __init__(self, label, size):
            self.lhs = label
            self.size = size
    class Literal:
        def __init__(self, val, size):
            self.lhs = val
            self.size = size
    class Void:
        def __init__(self, size):
            self.lhs = None
            self.size = size

    @staticmethod
    def parse(tstream_full):
        tstream = adapt_strip_newlines(tstream_full)
        lhs = next(tstream)
        ensure_match_token(next(tstream), "separator", ":")
        size = matching_unwrap_token(next(tstream), "literal")
        if lhs.ttype == "label":
            return BitGroup.Labeled(lhs.tval, size)
        elif lhs.ttype == "literal":
            return BitGroup.Literal(lhs.tval, size)
        elif lhs.ttype == "void":
            return BitGroup.Void(size)
        else:
            error_no_match(lhs)

def parse(filename):
    tokens = lexer.tokenize(filename)
    decoder = Decoder.parse(iter(tokens))
    return decoder