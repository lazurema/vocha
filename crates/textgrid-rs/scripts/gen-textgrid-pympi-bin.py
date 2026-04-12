import glob

from pympi import TextGrid

for entry_path in glob.glob("test-fixtures/**/*.TextGrid"):
    print(f"Processing `{entry_path}`…")

    with open(entry_path, "rb") as f:
        bom = f.read(2)
        if bom in (b"\xff\xfe", b"\xfe\xff"):
            codec = "utf-16"
        else:
            codec = "utf-8"

    textgrid = TextGrid(entry_path, codec=codec)
    textgrid.to_file(entry_path + ".pympi-bin", mode="binary")
