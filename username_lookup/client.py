# Copyright: https://git.bounceme.net/hex0x0000/BirthdayBot/src/branch/master/LICENSE
import socket
import sys

if __name__ == "__main__":
    client = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    client.connect(('0.0.0.0', 9894))
    username = sys.argv[1]
    client.sendall(len(username).to_bytes(2, "big"))
    client.sendall(username.encode(encoding="utf8"))
    print(int.from_bytes(client.recv(8), "big", signed=True))
    client.close()
