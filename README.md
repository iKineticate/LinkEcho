![image](https://raw.githubusercontent.com/iKineticate/LinkEcho/refs/heads/master/screenshots/app.png)

# LinkEcho

一键批量更换、自定义或还原快捷方式图标，让您的桌面焕然一新。

- **极简操作**：选择匹配图标即可批量更换快捷方式图标，无需逐一手动设置  
- **图标自定**: 自由调节快捷方式原图标/新图标尺寸与圆角，支持纯色/渐变色背景层，亦可调节其尺寸与圆角
- **格式广泛**：支持 ICO/PNG/SVG/BMP/WEBP/TIFF/EXE 格式
- **场景支持**：覆盖桌面、开始菜单及任意文件夹中的快捷方式  
- **无损还原**：随时一键恢复快捷方式图标默认状态

# 使用

## 🔒 管理员权限说明

**为何需要管理员权限？**  
`所有用户文件夹`、`开始菜单`、`任务栏`等位置的快捷方式受Windows权限保护，Windows要求临时提升权限方可修改快捷方式属性。

**安全承诺**  
✅ 不收集任何数据  
✅ 无网络传输行为  
✅ 权限仅用于修改快捷方式图标路径

## ✨ 功能介绍

<details>
<summary>1. 一键更换所有图标</summary>
#### 图标匹配规则
- **格式支持**：`ICO` `PNG` `SVG` `BMP` `WEBP` `TIFF` `EXE`
- **智能匹配**：图标文件需满足以下条件之一：
```bash
# 精确匹配（最高优先级）
快捷方式名 = "Visual Studio" → 图标名 = "Visual Studio.png"

# 包含匹配（次要优先级）
快捷方式名 = "Chrome" → 图标名 = "Chrome Beta.ico"
快捷方式名 = "Chrome Canary" -> 图标名 = "Chrome"
```

> [!WARNING]
> **UWP/WSA 应用限制**：
>  ```diff
>  - 更换后无法通过本工具恢复默认图标
>  + 恢复方法：需手动删除快捷方式并重新创建
>  ```
</details>

---

### 2. 图标还原
- **普通快捷方式**：通过「恢复图标」按钮恢复 或「恢复所有图标」按钮一键恢复
- **UWP/WSA 快捷方式**：需手动重建快捷方式（「工具」-「创建应用快捷方式」）

### 3. 自定义图标

---

<p>
    <p align="center" >
      <img src="https://raw.githubusercontent.com/DioxusLabs/dioxus/8b4ccb7817a98379eff2f348f55bbd84f9f12ea3/notes/splash-header-darkmode.svg" style="width: 80%; height: auto;">
      <br>
    </p>
</p>