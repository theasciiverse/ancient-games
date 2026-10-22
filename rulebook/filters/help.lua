-- The game's F1 help (HELP.txt): only the short form of the rules, the
-- fenced divs of class "brief", each under the heading of the section it
-- is in, underlined as in RULES.txt so the game finds the rule set's
-- section. A brief before any section comes first, under no heading.
function Pandoc(doc)
  local out = {}
  local heading
  for _, block in ipairs(doc.blocks) do
    if block.t == "Header" and block.level == 2 then
      heading = pandoc.utils.stringify(block)
    elseif block.t == "Header" and block.level == 1 then
      heading = nil
    elseif block.t == "Div" and block.classes:includes("brief") then
      if heading then
        local mark = string.rep("-", utf8.len(heading))
        table.insert(out, pandoc.RawBlock("plain", heading .. "\n" .. mark))
        heading = nil
      end
      for _, b in ipairs(block.content) do
        table.insert(out, b)
      end
    end
  end
  doc.blocks = out
  return doc
end
