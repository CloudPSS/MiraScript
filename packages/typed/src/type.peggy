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

function literal(value) {
  return { kind: "literal", value };
}

function interpolated(type) {
  return { __type: type };
}

function string(parts) {
  const result = [];
  for (const part of parts) {
    if (part.kind === 'literal' && typeof part.value == 'string') {
      if (result.length > 0 && result[result.length - 1].kind === 'literal') {
        result[result.length - 1].value += part.value;
      } else {
        result.push({ kind: 'literal', value: part.value });
      }
    } else {
      result.push(part);
    }
  }
  if (result.length === 0) return '';
  if (result.length === 1 && result[0].kind === 'literal') {
    return result[0].value;
  }
  return { kind: 'template', parts: result };
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
  = FunctionType
  / name:Identifier generic:(_ "<" _ @TypeParamList _ ">")? {
      if (name === 'fn') throw new Error("'fn' is a reserved keyword");
      if (generic) {
        if (name === 'array') {
          if (generic.length !== 1) throw new Error("'array' expects one generic type parameter");
          return array(generic[0]);
        }
        if (name === 'record') {
          if (generic.length !== 1) throw new Error("'record' expects one generic type parameter");
          return { kind: "record", fields: [], value: generic[0] };
        }
        throw new Error(`'${name}' is not a generic type`);
      }
      if (name === 'true') return { kind: "literal", value: true };
      if (name === 'false') return { kind: "literal", value: false };
      return named(name);
    }
  / lit:String {
      if (typeof lit == 'string') return { kind: "literal", value: lit };
      return lit;
    }
  / "(" _ @Type _ ")"
  / RecordType

///////////////////////////////////////////////////////////////////////////////
// Function
///////////////////////////////////////////////////////////////////////////////

FunctionType
  = "fn" typeParams:(_ "<" _ @TypeParamList _ ">")? _ "(" _ params:FunctionParamList? _ ")" ret:(_ "->" _ @Type)? {
      const result = { kind: "function", params: params ?? [] };
      if (typeParams !== null) result.typeParams = typeParams.map(name => Symbol(name));
      if (ret !== null) result.returns = ret;
      return result;
    }

TypeParamList
  = @Identifier|1.., _ "," _ | ( _ "," _ )?

FunctionParamList
  = params:FunctionParam|1.., _ "," _ | ( _ "," _ )? {
      for (let i = 0; i < params.length - 1; i++) {
        if (params[i].spread) throw new Error("Rest parameter must be last");
      }
      return params;
    }

FunctionParam
  = spread:".." name:Identifier? _ t:(":" _ @Type)? {
      return {
        name: name ?? "",
        spread: true,
        type: t ?? { kind: "array", element: "any" }
      }
    }
  / name:Identifier _ t:(":" _ @Type)? {
      return { name, type: t ?? "any" };
    }

///////////////////////////////////////////////////////////////////////////////
// Record
///////////////////////////////////////////////////////////////////////////////

RecordType
  = "(" _ fields:FieldList? _ ")" { return record(fields ?? []); }

FieldList
  = @Field|1.., _ "," _ | ( _ "," _ )?

Field
  = name:FieldName opt:("?"?) _ ":" _ t:Type {
      return {
        name,
        optional: opt != null,
        type: t
      };
    }
  / type:Type { return { type }; }

FieldName
  = Identifier
  / name:String {
      if (typeof name != 'string') throw new Error('Field name must be a string literal');
      return name;
    }

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
  = '"' parts:DoubleQuotePart* '"' { return string(parts); }
  / "'" parts:SingleQuotePart* "'" { return string(parts); }
  / "`" parts:BacktickPart* "`" { return string(parts); }

DoubleQuotePart
  = "$" "(" _ @Type _ ")"
  / chars:DoubleQuoteChar+ { return literal(chars.join('')); }

SingleQuotePart
  = "$" "(" _ @Type _ ")"
  / chars:SingleQuoteChar+ { return literal(chars.join('')); }

BacktickPart
  = "$" "(" _ @Type _ ")"
  / chars:BacktickChar+ { return literal(chars.join('')); }

DoubleQuoteChar
  = EscapeSequence
  / [^\\$"]

SingleQuoteChar
  = EscapeSequence
  / [^\\$']

BacktickChar
  = EscapeSequence
  / [^\\$`]

Identifier
  = $(("_"+ / "@"+ / "$"+ / [\p{XID_Start}]u) [\p{XID_Continue}]u*)

_
  = [ \t\v\f\r\n]*