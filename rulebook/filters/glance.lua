-- The "at a glance" table right under each chapter's title becomes a box:
-- framed in the PDF, of class "glance" in the HTML. Other formats keep a
-- plain table.
local BOX_OPEN = [[
#block(width: 100%, fill: rgb("#f4ecdc"), stroke: 0.7pt + rgb("#b5432a"),
  radius: 4pt, inset: (x: 12pt, y: 4pt), above: 0.4cm, below: 0.8cm)[
#show table.cell.where(x: 0): smallcaps
]]

function Pandoc(doc)
  local typst = FORMAT:match("typst")
  local html = FORMAT:match("html")
  if not (typst or html) then return nil end
  local out = {}
  -- In a one-chapter page the chapter's heading is the title, so the
  -- first table before any heading counts too.
  local after_title = true
  for _, block in ipairs(doc.blocks) do
    if block.t == "Header" then
      after_title = block.level == 1
      table.insert(out, block)
    elseif block.t == "Table" and after_title then
      after_title = false
      if typst then
        table.insert(out, pandoc.RawBlock("typst", BOX_OPEN))
        table.insert(out, block)
        table.insert(out, pandoc.RawBlock("typst", "]"))
      else
        block.classes:insert("glance")
        table.insert(out, block)
      end
    else
      if block.t ~= "Para" or #block.content > 0 then
        after_title = after_title and block.t == "Plain"
      end
      table.insert(out, block)
    end
  end
  doc.blocks = out
  return doc
end
