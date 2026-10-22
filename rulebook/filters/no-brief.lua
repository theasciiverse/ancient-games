-- The short form of the rules (fenced divs of class "brief", written for
-- the game's F1 help) is left out of the book and of RULES.txt, which
-- carry the full rules.
function Div(div)
  if div.classes:includes("brief") then
    return {}
  end
end
