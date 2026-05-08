import json
from dataclasses import dataclass, field
from typing import List, Tuple, Optional, Any, Dict

@dataclass
class Position:
    offset: int
    line: int
    col: int

    def to_dict(self):
        return {"offset": self.offset, "line": self.line, "col": self.col}

@dataclass
class Span:
    start: Position
    end: Position

    def to_dict(self):
        return {"start": self.start.to_dict(), "end": self.end.to_dict()}

@dataclass
class Comment:
    text: str
    span: Span

@dataclass
class Unit:
    id: int
    offset: int

@dataclass
class Container:
    id: int
    contents: List[Unit] = field(default_factory=list)
    error_code: Optional[str] = None

@dataclass
class Diagnostic:
    message: str
    offset: int

@dataclass
class ParserResult:
    root_id: int
    containers: Dict[int, Container]
    symbols: Dict[str, int]
    comments: List[Comment]
    diagnostics: List[Diagnostic]

def sweep(text: str, builtins: Dict[str, int] = None, constants: Dict[str, int] = None) -> ParserResult:
    builtins = builtins if builtins is not None else {
        "+": -1, "-": -2, "*": -3, "/": -4, "=": -5, "^": -6,
        ",": -7, ";": -8, "\n": -9
    }
    constants = constants or {}
    
    symbols: Dict[str, int] = {}
    symbols.update(builtins)
    symbols.update(constants)
    
    next_id = 1
    containers: Dict[int, Container] = {}
    
    def get_symbol_id(sym: str) -> int:
        nonlocal next_id
        if sym in symbols:
            return symbols[sym]
        sid = next_id
        symbols[sym] = sid
        next_id += 1
        return sid

    def create_container() -> int:
        nonlocal next_id
        cid = next_id
        next_id += 1
        containers[cid] = Container(id=cid)
        return cid

    root_id = 0
    containers[root_id] = Container(id=root_id)
    
    current_line_id = create_container()
    containers[root_id].contents.append(Unit(id=current_line_id, offset=0))
    stack = [root_id, current_line_id]
    
    active_sym = ""
    sym_start_offset = -1
    
    comments: List[Comment] = []
    diagnostics: List[Diagnostic] = []
    
    in_comment = False
    comment_text = ""
    comment_start_pos = None

    OPERATORS = "+-*/=^"
    SEPARATORS = " \t"

    i = 0
    line = 0
    col = 0
    
    def flush_sym():
        nonlocal active_sym, sym_start_offset
        if active_sym:
            sid = get_symbol_id(active_sym)
            containers[stack[-1]].contents.append(Unit(id=sid, offset=sym_start_offset))
            active_sym = ""
            sym_start_offset = -1

    while i < len(text):
        char = text[i]
        offset = i
        current_pos = Position(offset, line, col)

        if in_comment:
            if char == '\n':
                span = Span(comment_start_pos, current_pos)
                comments.append(Comment(comment_text, span))
                in_comment = False
                comment_text = ""
                # Do NOT continue; let \n act structurally to end the line/act as separator
            else:
                comment_text += char
                if char == '\n': # Fallback theoretically impossible here
                    line += 1
                    col = 0
                else:
                    col += 1
                i += 1
                continue

        is_slash_slash = char == '/' and i + 1 < len(text) and text[i+1] == '/'
        if not in_comment and (char == '#' or is_slash_slash):
            flush_sym()
            in_comment = True
            comment_start_pos = current_pos
            if is_slash_slash:
                i += 2
                col += 2
            else:
                i += 1
                col += 1
            continue

        if char == '(':
            flush_sym()
            new_cid = create_container()
            containers[stack[-1]].contents.append(Unit(id=new_cid, offset=offset))
            stack.append(new_cid)
            
        elif char == ')':
            flush_sym()
            if len(stack) > 2:
                stack.pop()
            else:
                diagnostics.append(Diagnostic("Stray closer ')'", offset))
                
        elif char in ('\n', ';'):
            flush_sym()
            if len(stack) == 2:
                # Root level line termination
                stack.pop()
                new_line_id = create_container()
                containers[root_id].contents.append(Unit(id=new_line_id, offset=offset))
                stack.append(new_line_id)
            else:
                # Nested sequence marker
                op_id = get_symbol_id(',')
                containers[stack[-1]].contents.append(Unit(id=op_id, offset=offset))
                
        elif char in SEPARATORS:
            flush_sym()
            
        elif char == ',':
            flush_sym()
            op_id = get_symbol_id(',')
            containers[stack[-1]].contents.append(Unit(id=op_id, offset=offset))
            
        elif char in OPERATORS:
            flush_sym()
            op_id = get_symbol_id(char)
            containers[stack[-1]].contents.append(Unit(id=op_id, offset=offset))
            
        else:
            if not active_sym:
                sym_start_offset = offset
            active_sym += char
            
        if char == '\n':
            line += 1
            col = 0
        else:
            col += 1
        i += 1
        
    flush_sym()
    if in_comment:
        span = Span(comment_start_pos, Position(len(text), line, col))
        comments.append(Comment(comment_text, span))

    while len(stack) > 2:
        cid = stack.pop()
        containers[cid].error_code = "UNCLOSED"

    return ParserResult(
        root_id=root_id,
        containers=containers,
        symbols=symbols,
        comments=comments,
        diagnostics=diagnostics
    )


# --- TDD TEST SUITE ---

def reconstruct(parser_result: ParserResult) -> List[Any]:
    """Helper to turn the flat topology back into lists for easy assertions."""
    reverse_symbols = {v: k for k, v in parser_result.symbols.items()}
    
    def resolve_container(cid: int) -> List[Any]:
        c = parser_result.containers.get(cid)
        if not c:
            return []
        res = []
        for unit in c.contents:
            if unit.id in parser_result.containers:
                res.append(resolve_container(unit.id))
            else:
                res.append(reverse_symbols.get(unit.id, f"?{unit.id}?"))
        return res
        
    return resolve_container(parser_result.root_id)

def run_tests():
    cases = [
        ("5", ([["5"]], [])),
        ("x = 5", ([["x", "=", "5"]], [])),
        ("3 + (1 + 2)", ([["3", "+", ["1", "+", "2"]]], [])),
        ("x = 5 # comment", ([["x", "=", "5"]], [" comment"])),
        ("x=1; y=2", ([["x", "=", "1"], ["y", "=", "2"]], [])),
        ("z = (3, 5\n 7)", ([["z", "=", ["3", ",", "5", ",", "7"]]], [])),
        ("z = (3; 5)", ([["z", "=", ["3", ",", "5"]]], [])),
        ("outer = (\n    1 + 1, // first\n    (2 + 2) # second\n)", 
         ([["outer", "=", [",", "1", "+", "1", ",", ",", ["2", "+", "2"], ","]]], 
          [" first", " second"])),
        ("x = (5 6)", ([["x", "=", ["5", "6"]]], [])),
        ("-5", ([["-", "5"]], [])),
    ]

    passed = 0
    print("Running Topology Tests...")
    for i, (input_text, (expected_root, expected_comment_texts)) in enumerate(cases):
        result = sweep(input_text)
        root = reconstruct(result)
        
        # Remove trailing empty line containers for cleaner test matching
        while root and not root[-1]:
            root.pop()
            
        comment_texts = [c.text for c in result.comments]
        
        if root == expected_root and comment_texts == expected_comment_texts:
            print(f"✅ Test {i} passed")
            passed += 1
        else:
            print(f"❌ Test {i} failed!")
            print(f"   Input:    {repr(input_text)}")
            print(f"   Expected: {expected_root}")
            print(f"   Result:   {root}")
            print(f"   Comments: {comment_texts}")

    print("\nRunning Diagnostics Tests...")
    err_cases = [
        ("x = (5", "UNCLOSED"),
        ("x = 5)", "Stray closer")
    ]
    for i, (input_text, expected_err) in enumerate(err_cases):
        result = sweep(input_text)
        found = False
        if "UNCLOSED" in expected_err:
            for c in result.containers.values():
                if c.error_code == "UNCLOSED":
                    found = True
                    break
        elif "Stray" in expected_err:
            if any("Stray" in d.message for d in result.diagnostics):
                found = True
        
        if found:
            print(f"✅ Error Test {i} passed")
            passed += 1
        else:
            print(f"❌ Error Test {i} failed!")
            
    print(f"\nSummary: {passed}/{len(cases) + len(err_cases)} passed")

if __name__ == "__main__":
    run_tests()
