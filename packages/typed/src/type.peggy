{
function named(name) {
  return name;
}

function array(element) {
  return { kind: "array", element };
}

function union(types) {
  if (types.length === 1) return types[0];
  return { kind: "union", types };
}

function record(fields) {
  for(let i = 0; i < fields.length; i++) {
    fields[i].name ??= String(i);
  }
  return { kind: "record", fields };
}
}

Start
  = _ @Type _

///////////////////////////////////////////////////////////////////////////////
// Type (union)
///////////////////////////////////////////////////////////////////////////////

Type
  = UnionType

UnionType
  = _ ( "|" _ )? u:PostfixType|1.., _ "|" _ | {
      return union(u);
    }

///////////////////////////////////////////////////////////////////////////////
// Array
///////////////////////////////////////////////////////////////////////////////

PostfixType
  = base:PrimaryType suffix:("[]" _)* {
      let t = base;
      for (const _ of suffix)
        t = array(t);
      return t;
    }

///////////////////////////////////////////////////////////////////////////////
// Primary
///////////////////////////////////////////////////////////////////////////////

PrimaryType
  = name:Identifier generic:(_ "<" _ @Type _ ">")? {
      if (generic) {
        if (name === 'array') return array(generic);
        if (name === 'record') return { kind: "record", fields: [], value: generic };
      }
      if (name === 'true') return { kind: "literal", value: true };
      if (name === 'false') return { kind: "literal", value: false };
      return named(name);
    }
  / lit:String {
      return { kind: "literal", value: lit };
    }
  / "(" _ @Type _ ")"
  / RecordType

///////////////////////////////////////////////////////////////////////////////
// Record
///////////////////////////////////////////////////////////////////////////////

RecordType
  = "(" _ ")" { return record([]); } 
  / "(" _ fields:FieldList _ ")" { return record(fields); }

FieldList
  = @Field|1.., _ "," _ | ( _ "," _ )?

Field
  = name:FieldName opt:("?"?) _ ":" _ t:Type {
      return {
        name,
        optional: opt !== null,
        type: t
      };
    }
  / type:Type { return { type }; }

FieldName
  = Identifier
  / String

///////////////////////////////////////////////////////////////////////////////
// Lexer
///////////////////////////////////////////////////////////////////////////////

HexDigit
  = [0-9a-f]i

EscapeSequence
  = "\\" char:(['"`\\nrtbfv0$]) {
    switch (char) {
      case 'n': return '\n';
      case 'r': return '\r';
      case 't': return '\t';
      case 'b': return '\b';
      case 'f': return '\f';
      case 'v': return '\v';
      case '0': return '\0';
      default: return char;
    }
  } / "\\x" hex:$(HexDigit HexDigit) {
    const code = parseInt(hex, 16);
    // Valid ASCII characters are in the range 0x00 to 0x7F.
    if (code >= 128) throw new Error(`Invalid hex escape sequence: \\x${hex}`);
    return String.fromCharCode(code);
  } / "\\u{" hex:$(HexDigit+) "}" {
    const code = parseInt(hex, 16);
    // Valid Unicode code points are in the range 0x0000 to 0x10FFFF, not including the surrogate halves (0xD800 to 0xDFFF). 
    if (code > 0x10FFFF || (code >= 0xD800 && code <= 0xDFFF)) {
      throw new Error(`Invalid Unicode code point: \\u{${hex}}`);
    }
    return String.fromCodePoint(code);
  }

String
  = '"' @$[^\\$"]* '"'
  / "'" @$[^\\$']* "'"
  / "`" @$[^\\$`]* "`"
  / '"' parts:( EscapeSequence / $[^\\$"]+ )+ '"' {
      return parts.join('');
    }
  / "'" parts:( EscapeSequence / $[^\\$']+ )+ "'" {
      return parts.join('');
    }
  / "`" parts:( EscapeSequence / $[^\\$`]+ )+ "`" {
      return parts.join('');
    }

Identifier
  = $(("_"+ / "@"+ / "$"+ / [\p{XID_Start}]u) [\p{XID_Continue}]u*)

_
  = [ \t\v\f\r\n]*