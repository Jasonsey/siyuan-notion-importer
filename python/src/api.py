# -*- coding: utf-8 -*-
#
# siyuan-notion-import-update
#
# @Author: Lin, Max
# @Email : jason.max.lin@outlook.com
# @Time  : 2025/2/10 11:48
#
# =============================================================================
"""api.py"""
from asyncio import Semaphore
from typing import *
from dataclasses import dataclass, field
import json
from pathlib import Path

import aiohttp


@dataclass
class Api:
    notebook_name: str = 'notion'
    data_home: str = "/Users/max/SiYuan/data"
    base_url: str = "http://127.0.0.1:6806"
    headers: Dict[str, str] = field(default_factory=lambda: {"Content-Type": "application/json"})
    _notebook_home: Optional[str] = None
    _sem: Semaphore = Semaphore(500)

    async def notebook_home(self):
        if self._notebook_home is None:
            notebooks = await self.list_notebooks()
            for notebook in notebooks:
                if notebook['name'] == self.notebook_name:
                    self._notebook_home = str(Path(self.data_home) / notebook['id'])
            if self._notebook_home is None:
                raise RuntimeError(f"No notebook named {self.notebook_name}")
        return self._notebook_home

    async def get_all_sy_files(self):
        path = Path(await self.notebook_home())
        names = list(path.glob("**/*.sy"))
        return names

    async def list_notebooks(self):
        async with self._sem:
            async with aiohttp.ClientSession(headers=self.headers) as session:
                url = f"{self.base_url}/api/notebook/lsNotebooks"
                async with session.post(url) as response:
                    res = await response.json()
        if res['code'] != 0:
            raise RuntimeError(f"Error listing notebooks: {res['msg']}")
        else:
            return res['data']['notebooks']

    async def get_filepath_by_id(self, idx: str):
        async with self._sem:
            async with aiohttp.ClientSession(headers=self.headers) as session:
                url = f"{self.base_url}/api/filetree/getPathByID"
                payload = json.dumps({"id": idx})
                async with session.post(url, data=payload) as response:
                    res = await response.json()
        if res['code'] == 0:
            # success
            path = Path(await self.notebook_home()) / ('.' + res['data'])
            return path
        else:
            raise ValueError(res['msg'])

    async def update_block(self, data: str, idx: str):
        """使用markdown更新块内容"""
        async with self._sem:
            async with aiohttp.ClientSession(headers=self.headers) as session:
                url = f"{self.base_url}/api/block/updateBlock"
                payload = json.dumps({"data": data, "dataType": "markdown", "id": idx})
                async with session.post(url, data=payload) as response:
                    res = await response.json()
        if res['code'] != 0:
            raise RuntimeError(f"Error updating block: {res['msg']}")

    async def get_block_kramdown(self, idx: str):
        async with self._sem:
            async with aiohttp.ClientSession(headers=self.headers) as session:
                url = f"{self.base_url}/api/block/getBlockKramdown"
                payload = json.dumps({"id": idx})
                async with session.post(url, data=payload) as response:
                    res = await response.json()
        if res['code'] != 0:
            raise RuntimeError(f"Error getting block kramdown: {res['msg']}")
        else:
            return res['data']["kramdown"]

    async def insert_block(self, data: str, next_id="", previous_id="", parent_id=""):
        """插入markdown格式的块"""
        async with self._sem:
            async with aiohttp.ClientSession(headers=self.headers) as session:
                url = f"{self.base_url}/api/block/insertBlock"
                payload = json.dumps({"data": data, "nextID": next_id, "previousID": previous_id, "parentID": parent_id, "dataType": "markdown"})
                async with session.post(url, data=payload) as response:
                    res = await response.json()
        if res['code'] != 0:
            raise RuntimeError(f"Error inserting block: {res['msg']}")
        else:
            return res['data']['doOperations']['id']

    async def delete_block(self, idx: str):
        async with self._sem:
            async with aiohttp.ClientSession(headers=self.headers) as session:
                url = f"{self.base_url}/api/block/deleteBlock"
                payload = json.dumps({"id": idx})
                async with session.post(url, data=payload) as response:
                    res = await response.json()
        if res['code'] != 0:
            raise RuntimeError(f"Error deleting block: {res['msg']}")

    async def get_child_blocks(self, idx: str):
        async with self._sem:
            async with aiohttp.ClientSession(headers=self.headers) as session:
                url = f"{self.base_url}/api/block/getChildBlocks"
                payload = json.dumps({"id": idx})
                async with session.post(url, data=payload) as response:
                    res = await response.json()
        if res['code'] != 0:
            raise RuntimeError(f"Error getting child blocks: {res['msg']}")
        else:
            return res['data']
