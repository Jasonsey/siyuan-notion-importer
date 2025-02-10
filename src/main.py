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
from tqdm import tqdm

from api import Api


def update_node_paragraph(data: str):
    """更新inline math"""
    data = "\n".join(data.split("\n")[:-1])
    if '\\$' in data:
        data = re.sub(r"\\(\\*)", r"\1", data)
    # img optimize show
    data = re.sub(r"!?\[(.*?)]\((.*?)\.(bmp|jpg|png|tif|gif|pcx|tga|exif|fpx|svg|psd|cdr|pcd|dxf|ufo|eps|ai|raw|WMF|webp|jpeg)\)", r"![\1](\2.\3)", data)
    return data


def update_node_math_block(data: str):
    data = "\n".join(data.split("\n")[:-1])
    if "$$\n$" in data:
        data = re.sub(r"\$\$\n\$*(.*?)\$*\n\$\$", r"$$\n\1\n$$", data, flags=re.DOTALL)
    return data


def update_node_blockquote(data: str):
    if "[!important]" in data or "[!info]" in data:
        res = []

        important_data = re.findall(r"> \[!important](.*?)(?:\[!.*?]|{: id=.*? updated=.*?})", data, flags=re.DOTALL)
        important_data = [re.sub(r"\n?(.*?)(?:\n> )?$", r"\1", item) for item in important_data]
        res.extend(important_data)

        info_data = re.findall(r"> \[!info](.*?)(?:\[!.*?]|{: id=.*? updated=.*?})", data, flags=re.DOTALL)
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


def update_data(data: dict):
    """更新NodeParagraph"""
    api = Api()

    if data['Type'] == "NodeParagraph":
        # update
        idx = data['ID']
        markdown_data = api.get_block_kramdown(idx)
        markdown_data = update_node_paragraph(markdown_data)
        api.update_block(markdown_data, idx=idx)
    elif data['Type'] == "NodeMathBlock":
        # update math block
        idx = data['ID']
        markdown_data = api.get_block_kramdown(idx)
        markdown_data = update_node_math_block(markdown_data)
        api.update_block(markdown_data, idx=idx)
    elif data['Type'] == "NodeBlockquote":
        # update quote
        idx = data['ID']
        markdown_data = api.get_block_kramdown(idx)
        markdown_data = update_node_blockquote(markdown_data)
        api.update_block(markdown_data, idx=idx)
    elif "Children" not in data:
        pass
    else:
        for child in data['Children']:
            update_data(child)


def main():
    api = Api()

    path = api.get_filepath_by_id("20250210140717-bh2fko2")
    data = json.load(open(path))
    update_data(data)

    data = json.load(open(path))
    print(json.dumps(data, indent=2, ensure_ascii=False))


def main2():
    api = Api()
    files = api.get_all_sy_files()
    for file in tqdm(files):
        data = json.load(open(file))
        update_data(data)


if __name__ == "__main__":
    main2()
