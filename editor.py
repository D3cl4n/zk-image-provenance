import struct
import zlib
import io
from PIL import Image

'''
- This script aims to demonstrate editor functionality
- The editor recomputes the hash Poseidon(r||g||b||exif) to send to verifier as separate input
- TODO: rewrite this to read image bytes directly and only greyscale pixels from IDATA chunk
    - need to save all bytes 
'''
    

# class for all functionality related to the editor
class Editor:
    def __init__(self, image_path):
        self.image_path = image_path
        self.chunks = {} # keys is chunk types, value is data
        self.greyscale_chunks = {} # for edited chunks only
        self.r_coeff = 30
        self.g_coeff = 58
        self.b_coeff = 11
        self.r = []
        self.g = []
        self.b = []
        self.greyscale_pixels = []
        self.target = "greyscale.png"


    # helper function to read a chunk
    def read_chunk(self, f):
        chunk_length, chunk_type = struct.unpack(">I4s", f.read(8))
        chunk_data = f.read(chunk_length)
        expected_crc = zlib.crc32(chunk_type + chunk_data) & 0xffffffff
        actual_crc, = struct.unpack(">I", f.read(4))

        # check the crc for data corruption
        assert expected_crc == actual_crc

        return chunk_type, chunk_data


    # parse the png and save chunks in separate buffers, consolidate pixels to one buffer
    # follows this guide https://pyokagan.name/blog/2019-10-14-png/ 
    # TODO: change this to be able to concatenate multiple IDAT blocks
    def parse_png(self):
        print("[*] Parsing png chunks")

        png_signature = b"\x89\x50\x4E\x47\x0D\x0A\x1A\x0A"
        with open(self.image_path, "rb") as f:
            actual_signature = f.read(8)
            assert actual_signature == png_signature

            # parse all chunks out of the file
            while True:
                chunk_type, chunk_data = self.read_chunk(f)
                self.chunks[chunk_type] = chunk_data
                if chunk_type == b"IEND":
                    break

        print(f"[*] Read chunks: {list(self.chunks.keys())}")

    
    # greyscale the pixels in the IDAT chunk
    def greyscale(self):
        print("[*] Greyscaling pixels in png")
        image = Image.open(self.image_path).convert("RGB")
        width, height = image.size
        pixels = image.load()

        new_chunks = {}
        temp.read(8) # skip the png signature

        while True:
            chunk_type, chunk = self.read_chunk(temp)
            if chunk_type == None:
                break

            new_chunks[chunk_type] = chunk

            if chunk_type == b"IEND":
                break

        return new_chunks


    # reassamble and save the edited png based on dictionary of chunks after greyscaling
    def reassemble_png(self):
        print(f"[*] Reassembling edited png as: {self.target}")

        png_signature = b"\x89\x50\x4E\x47\x0D\x0A\x1A\x0A"
        with open(self.target, "wb") as f:
            f.write(png_signature)
            f.write(self.greyscale_chunks[b"IHDR"])
            f.write(self.greyscale_chunks[b"IDAT"])
            f.write(self.chunks[b"eXIf"])
            f.write(self.chunks[b"hASh"])
            f.write(self.chunks[b"sIGn"])
            f.write(self.greyscale_chunks[b"IEND"])

        print(f"[*] Saved greyscaled image with preserved custom chunks to {self.target}")


# main function
def main():
    # editor functionality
    editor = Editor("original.png")
    editor.parse_png()
    editor.greyscale_chunks = editor.greyscale()
    editor.reassemble_png()


if __name__ == '__main__':
    main()