-- A code block of class "example" with file="<package>/<name>" shows that
-- worked example, as rulebook/build.sh wrote it to examples/ from the game.
function CodeBlock(block)
  if not block.classes:includes("example") then return nil end
  local name = block.attributes.file
  local f = io.open("examples/" .. name .. ".txt", "r")
  if not f then
    error("no worked example " .. name .. " (is it in pictures.txt?)")
  end
  local text = f:read("a"):gsub("\n$", "")
  f:close()
  return pandoc.CodeBlock(text, { class = "example" })
end
