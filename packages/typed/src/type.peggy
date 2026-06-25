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
  = _ t:Type _ { return t; }

///////////////////////////////////////////////////////////////////////////////
// Type (union)
///////////////////////////////////////////////////////////////////////////////

Type
  = head:UnionType {
      return head;
    }

UnionType
  = _ "|" _ first:PostfixType rest:(_ "|" _ PostfixType)* {
      return union([
        first,
        ...rest.map(x => x[3])
      ]);
    }
  / first:PostfixType rest:(_ "|" _ PostfixType)+ {
      return union([
        first,
        ...rest.map(x => x[3])
      ]);
    }
  / PostfixType

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
  = name:Identifier generic:(_ "<" _ t:Type _ ">")? {
      if (generic) {
        const t = generic[3];
        if (name === 'array') return array(t);
        if (name === 'record') return { kind: "record", fields: [], value: t };
      }
      if (name === 'true') return { kind: "literal", value: true };
      if (name === 'false') return { kind: "literal", value: false };
      return named(name);
    }
  / lit:String {
      return { kind: "literal", value: lit };
    }
  / "(" _ t:Type _ ")" {
      return t;
    }
  / RecordType

///////////////////////////////////////////////////////////////////////////////
// Record
///////////////////////////////////////////////////////////////////////////////

RecordType
  = "(" _ ")" { return record([]); } 
  / "(" _ fields:FieldList _ ")" { return record(fields); }

FieldList
  = fields:Field|1.., _ "," _ | ( _ "," _ )? {
      return fields;
    }

Field
  = name:FieldName opt:("?"?) _ ":" _ t:Type {
      return {
        name,
        optional: opt !== null,
        type: t
      };
    }
  / t:Type {
      return {
        type: t
      };
    }

FieldName
  = Identifier
  / String

///////////////////////////////////////////////////////////////////////////////
// Lexer
///////////////////////////////////////////////////////////////////////////////

String
  = '"' @$[^"]* '"'
  / "'" @$[^']* "'"

Identifier
  = $([a-zA-Z_][a-zA-Z0-9_]*)

_
  = [ \t\r\n]*