import struct
import zlib
from PIL import Image


# class for PNG parsing functionality
class PNGUtils:
    def __init__(self, image_path):
        self.image = image_path
        self.png_signature = b"\x89\x50\x4E\x47\x0D\x0A\x1A\x0A"
        # {chunk_type : (length, chunk_type, data, crc)}
        self.chunks = {} 


    # helper function to read a single chunk from the image
    def read_single_chunk(self, f):
        chunk_length, chunk_type = struct.unpack(">I4s", f.read(8))
        chunk_data = f.read(chunk_length)
        expected_crc = zlib.crc32(chunk_type + chunk_data) & 0xffffffff
        actual_crc, = struct.unpack(">I", f.read(4))

        # check the crc for data corruption
        assert expected_crc == actual_crc

        return chunk_type, chunk_data, actual_crc


    # read all chunks from the image
    def read_all_chunks(self):
        print(f"[*] Reading all chunks from: {self.image}")

        with open(self.image, "rb") as f:
            assert self.png_signature == f.read(8)

            # read chunks until we have read IEND
            while True:
                chunk_type, chunk_data, chunk_crc = self.read_single_chunk(f)
                self.chunks[chunk_type] = [len(chunk_data), chunk_type, chunk_data, chunk_crc]
                
                if chunk_type == b"IEND":
                    break

        print(f"[*] Read chunks: {list(self.chunks.keys())}")


    # flip the color type field of the IHDR chunk to 0 to indicate a greyscaled image
    def flip_color_type(self):
        print("[*] Setting color type field to 0 in IHDR")
        assert b"IHDR" in self.chunks

        # the color type is the 10th byte in the IHDR chunk
        chunk = self.chunks[b"IHDR"]
        chunk_data = bytearray(chunk[2])
        chunk_data[9] = 0x00
        self.chunks[b"IHDR"] = [chunk[0], chunk[1], chunk_data, chunk[3]]


    # reassemble the png given the dictionary of chunks
    def reassemble_png(self, target):
        print("[*] Reassembling the png from the dictionary of chunks")

        with open(target, "wb") as f:
            f.write(self.png_signature)
            for _, chunk_info in self.chunks.items():
                length, type, data, _ = chunk_info
                new_crc = zlib.crc32(type + data) & 0xffffffff
                f.write(struct.pack(">I", length))
                f.write(type)
                f.write(data)
                f.write(struct.pack(">I", new_crc))


# class for the editor functionality
class EditorUtils:
    def __init__(self, image_path):
        self.image = image_path
        self.target = "photos/greyscale.png"
        # coefficients are rounded to the nearest integer to work in a circuit
        self.r_coeff = 30
        self.g_coeff = 58
        self.b_coeff = 11
        self.png_utils = PNGUtils(self.image)


    # reconstruct the IDAT chunk after computing raw greyscale values
    def reconstruct_IDAT(self, raw_data):
        compressed_data = zlib.compress(raw_data)
        self.png_utils.chunks[b"IDAT"] = [len(compressed_data), b"IDAT", compressed_data, 0] # update crc in file reconstruction


    # greyscale the pixels in the IDAT chunk(s) - after IDAT chunks are aggregated
    def greyscale(self):
        print(f"[*] Applying greyscale transformation to: {self.image}")
        self.png_utils.read_all_chunks()
        self.png_utils.flip_color_type()

        # access the pixel channels
        image = Image.open(self.image).convert("RGB")
        width, height = image.size
        pixels = image.load()
        
        raw_data = bytearray()
        for y in range(height):
            raw_data.append(0) # add filter type 0 to each row
            for x in range(width):
                r, g, b = pixels[x, y]
                raw_data.append((self.r_coeff * r + self.g_coeff * g + self.b_coeff * b) // 100) # using integer coefficients not floats

        self.reconstruct_IDAT(raw_data)
        self.png_utils.reassemble_png(self.target)


# main function
def main():
    editor = EditorUtils("photos/original_64.png")
    editor.greyscale()


if __name__ == '__main__':
    main()