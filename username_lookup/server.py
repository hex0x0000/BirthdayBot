# Copyright: https://git.bounceme.net/hex0x0000/BirthdayBot/src/branch/master/LICENSE
from pyrogram import Client
import asyncio
import random
import socket
import json


async def handle_client(app: Client, client: socket.socket) -> None:
    global run
    loop = asyncio.get_event_loop()
    pack_len = int.from_bytes(await loop.sock_recv(client, 2), "big")
    username = (await loop.sock_recv(client, pack_len)).decode("utf8")
    print(f"Looking up: {username}")
    await asyncio.sleep(1 + random.random() * 4)
    user_id = 0
    try:
        user_id = (await app.get_users(username)).id
    except:
        pass
    await loop.sock_sendall(client, user_id.to_bytes(8, "big"))
    client.close()


async def run_server(settings) -> None:
    global run
    async with Client("lookup", api_id=settings["api_id"], api_hash=settings["api_hash"]) as app:
        server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        server.bind(('0.0.0.0', 9894))
        server.listen(8)
        server.setblocking(False)
        print("Server started successfully")

        loop = asyncio.get_event_loop()
        while True:
            client, addr = await loop.sock_accept(server)
            print(f"Handling {addr}")
            loop.create_task(handle_client(app, client))

if __name__ == "__main__":
    with open("config.json", "r") as f:
        settings = json.load(f)
    asyncio.run(run_server(settings))
