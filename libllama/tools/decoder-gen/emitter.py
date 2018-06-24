import parser

def literal_to_int(lit):
    if lit.startswith("0b"):
        return int(lit[2:], base=2)
    elif lit.startswith("0x"):
        return int(lit[2:], base=16)
    else:
        return int(lit)

class Constraint:
    def __init__(self, mask, equals):
        self.mask = mask
        self.equals = equals

    def __str__(self):
        return f"enc & 0x{self.mask:08X} == 0x{self.equals:08X}"

def definition_to_constraint(defn):
    mask = 0
    req = 0
    for bitgroup in defn.bitgroups:
        bg_size = literal_to_int(bitgroup.size)
        mask <<= bg_size
        req <<= bg_size
        if isinstance(bitgroup, parser.BitGroup.Literal):
            mask |= 2**bg_size - 1
            binary = literal_to_int(bitgroup.lhs)
            assert(binary <= mask)
            req |= binary
    return Constraint(mask, req)

def to_CamelCase(string):
    return "".join([s.title() for s in string.split('_')])

def generate(file):
    decoder = parser.parse(file)

    indentation = 0
    def pcode(s):
        print(' ' * indentation + s)
    def indent():
        nonlocal indentation
        indentation += 2
    def unindent():
        nonlocal indentation
        indentation -= 2

    pcode("#[allow(unused_parens)]")
    pcode(f"pub fn decode(enc: {decoder.ty}) -> InstFn {{")
    indent()
    for category in decoder.categories:
        string = ") || (".join([ str(definition_to_constraint(defn))
                                    for defn in category.definitions ])
        pcode(f"if ({string}) {{")
        indent()

        for instr in category.instructions:
            constraint = definition_to_constraint(instr.defn)
            pcode(f"if {str(constraint)} {{")
            indent()

            pcode(f"return unsafe {{ ::std::mem::transmute(interpreter::{instr.name} as usize) }}")

            unindent()
            pcode("}")

        unindent()
        pcode("}")
    pcode("interpreter::undef")
    unindent()
    pcode("}")

    for category in decoder.categories:
        for instr in category.instructions:
            pcode(f"bitfield!({to_CamelCase(instr.name)}: {decoder.ty}, {{")
            indent()

            pos = 0
            num_labeled = sum((1 for b in instr.defn.bitgroups if isinstance(b, parser.BitGroup.Labeled)))
            labeled_i = 0
            for bitgroup in reversed(instr.defn.bitgroups):
                comma = "," if labeled_i != num_labeled - 1 else ""
                size = literal_to_int(bitgroup.size)
                if isinstance(bitgroup, parser.BitGroup.Labeled):
                    name = bitgroup.lhs
                    start = pos
                    pcode(f"{name}: {start}usize => {start + size - 1}usize{comma}") 
                    labeled_i += 1
                pos += size

            unindent()
            pcode("});")
