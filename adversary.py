import struct
import zlib
import ecdsa
from PIL import Image


# class for PNG parsing and I/O
class PNGUtils:
    def __init__(self, image):
        self.image = image
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
    

    # read all chunks from the PNG
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


# class for the adversary functionality 
class Adversary:
    def __init__(self, image):
        self.image = image
        self.png_utils = PNGUtils(self.image)
        self.sk = ecdsa.SigningKey.generate(curve=ecdsa.SECP256k1)
        self.vk = self.sk.get_verifying_key()

    
    # construct a chunk for embedding in the target image
    def construct_chunk(self, chunk_type, chunk_data):
        print(f"[*] Constructing a {chunk_type} chunk")

        chunk_length = struct.pack(">I", len(chunk_data))
        chunk_crc = struct.pack(">I", zlib.crc32(chunk_type + chunk_data) & 0xffffffff)

        return chunk_length + chunk_type + chunk_data + chunk_crc
    

    # make a chunk into a list for embedding properly
    def make_chunk_tuple(self, chunk_type, chunk_data):
        crc = zlib.crc32(chunk_type + chunk_data) & 0xffffffff

        return [len(chunk_data), chunk_type, chunk_data, crc]


    # reconstruct the image given the edited chunks
    def reconstruct_image(self, chunks, new_image):
        print(f"[*] Reconstructing chunks into: {new_image}")

        with open(new_image, "wb") as f:
            f.write(self.png_utils.png_signature)
            for _, chunk_info in chunks.items():
                length, type, data, _ = chunk_info
                new_crc = zlib.crc32(type + data) & 0xffffffff
                f.write(struct.pack(">I", length))
                f.write(type)
                f.write(data)
                f.write(struct.pack(">I", new_crc))


    # signature swapping attack
    def signature_swap(self):
        print("[*] Resigning PNG using non-trusted device signing key")
        print("[*] Signing original hash from PNG")

        self.png_utils.read_all_chunks()
        original_hash = self.png_utils.chunks[b"hASh"][2]
        signature = self.sk.sign_digest_deterministic(original_hash)
        modified_signature_chunk = self.make_chunk_tuple(b"sIGn", signature)

        # swap the original signature chunk for the forged one and reconstruct image
        self.png_utils.chunks[b"sIGn"] = modified_signature_chunk
        self.reconstruct_image(self.png_utils.chunks, "original_sig_swap.png")


    # hash swapping attack
    def hash_swap(self):
        print("[*] Swapping the hASh embedded in original png")

        self.png_utils.read_all_chunks()
        original_hash = self.png_utils.chunks[b"hASh"][2]
        modified_hash = b"\xAA" + original_hash[1:] # modify the first byte and confirm inequality

        assert modified_hash != original_hash

        modified_hash_chunk = self.make_chunk_tuple(b"hASh", modified_hash)
        # swap the original hash chunk of the forged one and reconstruct image
        self.png_utils.chunks[b"hASh"] = modified_hash_chunk
        self.reconstruct_image(self.png_utils.chunks, "original_hash_swap.png")


# main function
def main():
    adversary = Adversary("original.png")
    adversary.signature_swap()
    adversary.hash_swap()


if __name__ == '__main__':
    main()