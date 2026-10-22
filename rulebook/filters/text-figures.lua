-- Plain text output: a figure becomes its ASCII drawing (the .txt next to
-- the .svg) followed by its caption, and section headings are in capitals.
local function read(path)
  local f = io.open(path, "r")
  if not f then return nil end
  local s = f:read("a")
  f:close()
  return s
end

function Figure(fig)
  local src
  fig:walk({ Image = function(img) src = src or img.src end })
  if not src then return nil end
  local drawing = read((src:gsub("%.svg$", ".txt")))
  if not drawing then return nil end
  local out = { pandoc.CodeBlock((drawing:gsub("\n$", ""))) }
  for _, block in ipairs(fig.caption.long) do
    table.insert(out, block)
  end
  return out
end

-- Headings stand out in plain text: section headings in capitals.
function Header(h)
  -- (Before --shift-heading-level-by, so the sections are still level 2.)
  if h.level <= 2 then
    return h:walk({ Str = function(s) s.text = pandoc.text.upper(s.text); return s end })
  end
end
