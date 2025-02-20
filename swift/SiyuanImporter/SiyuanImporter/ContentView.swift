//
//  ContentView.swift
//  SiyuanImporter
//
//  Created by Monster 林 on 2025/2/20.
//

import SwiftUI

// 核心步骤:
// 0. 安装siyuan
// 1. 通过siyuan-ext-importer完成notion数据导入
// 2. 确认base url, 上图下内容
// 3. 确认data home, 上图下内容
// 4. 获取notebook名称列表
// 5. 选择notebook
// 6. 需要操作的文件列表
// 7. 显示处理进度
struct ContentView: View {
    var body: some View {
        VStack {
            Image(systemName: "globe")
                .imageScale(.large)
                .foregroundStyle(.tint)
            Text("Hello, world!")
        }
        .padding()
    }
}

#Preview {
    ContentView()
}
