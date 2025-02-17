# -*- coding: utf-8 -*-
#
# siyuan-notion-import-update
#
# @Author: Lin, Max
# @Email : jason.max.lin@outlook.com
# @Time  : 2025/2/10 11:35
#
# =============================================================================
"""main"""
import re
import json
from typing import Optional

import asyncio
from tqdm import tqdm

from api import Api

# change this if you need
api = Api(notebook_name="notion", data_home="/Users/max/SiYuan/data")


def update_node_paragraph(data: str):
    """更新inline math"""
    data = "\n".join(data.split("\n")[:-1])
    if '\\$' in data:
        data = re.sub(r"\\(\\*)", r"\1", data)
    # img optimize show
    data = re.sub(
        r"!?\[(.*?)]\((.*?)\.(bmp|jpg|png|tif|gif|pcx|tga|exif|fpx|svg|psd|cdr|pcd|dxf|ufo|eps|ai|raw|WMF|webp|jpeg)\)",
        r"![\1](\2.\3)", data)
    return data


def update_node_math_block(data: str):
    data = "\n".join(data.split("\n")[:-1])
    if "$$\n$" in data:
        data = re.sub(r"\$\$\n\$*(.*?)\$*\n\$\$", r"$$\n\1\n$$", data, flags=re.DOTALL)
    return data


def update_node_blockquote(data: str):
    if "[!important]" in data or "[!info]" in data:
        res = []

        important_data = re.findall(r"> \[!important](.*?)(?:\[!.*?]|{: .*?=.*? .*?=.*?})", data, flags=re.DOTALL)
        important_data = [item.strip() for item in important_data]
        res.extend(important_data)

        info_data = re.findall(r"> \[!info](.*?)(?:\[!.*?]|{: .*?=.*? .*?=.*?})", data, flags=re.DOTALL)
        info_data = [re.sub(r"\n?(.*?)(?:\n> )?$", r"\1", item) for item in info_data]
        for idx, item in enumerate(info_data):
            # url
            title = item.split("\n")[0].strip()
            url = re.findall(r"\[.*?]\((.*?)\)", item)
            if url:
                info_data[idx] = f"[{title}]({url[0]})"
            else:
                info_data[idx] = f"{title}"
        res.extend(info_data)

        res = "\n\n".join(res)
        return res
    return data


async def update_data(data: dict):
    """更新NodeParagraph"""
    if data['Type'] == "NodeParagraph":
        # update
        idx = data['ID']
        markdown_data = await api.get_block_kramdown(idx)
        markdown_data = update_node_paragraph(markdown_data)
        await api.update_block(markdown_data, idx=idx)
    elif data['Type'] == "NodeMathBlock":
        # update math block
        idx = data['ID']
        markdown_data = await api.get_block_kramdown(idx)
        markdown_data = update_node_math_block(markdown_data)
        await api.update_block(markdown_data, idx=idx)
    elif data['Type'] == "NodeBlockquote":
        # update quote
        idx = data['ID']
        markdown_data = await api.get_block_kramdown(idx)
        markdown_data = update_node_blockquote(markdown_data)
        await api.update_block(markdown_data, idx=idx)
    elif "Children" not in data:
        pass
    else:
        for child in data['Children']:
            await update_data(child)


async def update_file_with_bar(data: dict, bar: Optional[tqdm] = None):
    await update_data(data)
    if bar is not None:
        bar.update()


async def main():
    path = await api.get_filepath_by_id("20250211145209-ncpw7tz")
    data = json.load(open(path))
    await update_data(data)

    data = json.load(open(path))
    print(json.dumps(data, indent=2, ensure_ascii=False))


async def main2():
    files = await api.get_all_sy_files()
    bar = tqdm(total=len(files), desc="updating file")
    tasks = []
    for file in files:
        data = json.load(open(file))
        task = asyncio.create_task(update_file_with_bar(data, bar))
        tasks.append(task)
    await asyncio.gather(*tasks)


if __name__ == "__main__":
    asyncio.run(main2())
