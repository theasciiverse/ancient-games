-- Plain text output: a figure becomes its ASCII drawing (the .txt next to
-- the .svg) followed by its caption, photographs are left out, and
-- section headings are underlined.
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
  -- A photograph has no drawing: the text leaves it out.
  local drawing = src:match("%.svg$") and read((src:gsub("%.svg$", ".txt")))
  if not drawing then return {} end
  local out = { pandoc.CodeBlock((drawing:gsub("\n$", ""))) }
  for _, block in ipairs(fig.caption.long) do
    table.insert(out, block)
  end
  return out
end

-- Headings stand out in plain text: underlined, = for the references
-- and - for sections. (Before --shift-heading-level-by, so the sections
-- are still level 2 here.)
function Header(h)
  -- The chapter's own heading: the title block already shows it.
  if h.level == 1 and h.identifier ~= "bibliography" then
    return {}
  end
  if h.level <= 2 then
    local text = pandoc.utils.stringify(h)
    local mark = h.level == 1 and "=" or "-"
    return pandoc.RawBlock("plain", text .. "\n" .. string.rep(mark, utf8.len(text)))
  end
end
