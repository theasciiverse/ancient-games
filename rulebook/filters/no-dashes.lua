-- No en or em dashes in any output: they have rendered as "??" in some
-- viewers. Runs after citeproc, so it also catches page ranges and the
-- like that the citation style writes. The build checks the result too.
local function plain(s)
  return (s:gsub("\u{2014}", "-"):gsub("\u{2013}", "-"))
end

function Str(el)
  el.text = plain(el.text)
  return el
end

function Code(el)
  el.text = plain(el.text)
  return el
end

function CodeBlock(el)
  el.text = plain(el.text)
  return el
end
