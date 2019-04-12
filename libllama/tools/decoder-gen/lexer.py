import re

token_regex = re.compile(
    " " +
    "|\\/\\/.*?$" + # Comment
    "|(?P<keyword>decoder|category|or)[ \\t$]" +
    "|(?P<separator>=|\\:|;)" +
    "|(?P<bracket>[[\\]{}])" +
    "|(?P<void>_)" +
    "|(?P<literal>(0b)?\\d+)" +
    "|(?P<label>\\D\\w*)" +
    "|(?P<garbage>.*)"
)

class LineInfo:
    def __init__(self, line, col):
        self.line = line
        self.col = col

    def __str__(self):
        return "L{}:{}".format(self.line, self.col)

class Token:
    def __init__(self, ttype, val, lineinfo):
        self.ttype = ttype
        self.tval = val
        self.lineinfo = lineinfo

def tokenize(filename):
    tokens = []

    with open(filename) as defs:
        for lineno, line in enumerate(defs):
            col = 0
            while col < len(line) - 1:
                match_obj = token_regex.match(line, col)
                lineinfo = LineInfo(lineno + 1, match_obj.start() + 1)
                if not match_obj:
                    break
                col = match_obj.end()

                assert col != -1
                token_match = match_obj.groupdict()
                token_match = [(k, v) for k, v in token_match.items() if v is not None]
                if not token_match:
                    continue
                elif len(token_match) != 1:
                    raise AssertionError("Impossible: matched {} token types at once at {}!"
                        .format(len(token_match), lineinfo))
                ttype, tval = token_match[0]

                if ttype == "garbage":
                    raise RuntimeError("Lexing error: found unexpected `{}` at {}".format(tval, lineinfo))
                
                tokens.append(Token(ttype, tval, lineinfo))
            tokens.append(Token("newline", "", LineInfo(lineno + 1, col + 1)))
    
    return tokens
