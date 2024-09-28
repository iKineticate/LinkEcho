![image](https://raw.githubusercontent.com/iKineticate/LinkEcho/refs/heads/master/screenshots/app.png)

# LinkEcho

LinkEcho 可以评估桌面、开始菜单及其他文件夹的快捷方式的安全风险，支持更换单个快捷方式的图标，甚至可以自动更换所有快捷方式的图标。同时，它也支持将这些图标恢复为默认状态。

LinkEcho can assess the security risks of shortcuts, allow the replacement of individual shortcut icons, and even automate the icon replacement for all shortcuts. It also supports restoring these icons to their default state and retrieving shortcut properties from the desktop, Start Menu, and other folders.

---

## 功能

* **安全风险评估**：橙色标记表示该快捷方式可能存在风险（某些木马可能通过向快捷方式写入命令来侵入）。
* **快捷方式有效性检查**：红色标记表示快捷方式或其某一属性无效。
* **更换指定快捷方式图标**：绿色标记表示快捷方式图标已被更换。
* **批量自动更换快捷方式图标**：选择图标目录后，可以根据名称匹配来更换图标，如`Chrome Canary`可自动选择名为`Chrome`的图标；如`悟空`可自动选择名为`黑神话：悟空`的图标
* **恢复快捷方式图标**：恢复指定或所有快捷方式的图标为默认状态。
* **获取快捷方式属性**：工作目录、目标路径、启动参数等。

## Features
* **Security Risk Assessment**: Orange warning for shortcuts that may pose a risk (some malware may write commands to shortcuts to gain access).
* **Shortcut Validation**: Red marking indicates that a shortcut or one of its properties is invalid.
* **Replace Specific Shortcut Icon**: A green mark indicates that the shortcut icon has been replaced.
* **Batch auto change shortcuts icons**: After selecting an icon directory, icons can be replaced based on name matching. For instance, `Chrome Canary` can automatically select the icon named `Chrome,` while `Edge` can also choose the icon named `Microsoft Edge`.
* **Restore Shortcut Icon**: Restore specific or all shortcut icons to default.

*Windows 终端* 无法可视化图标，如需更丰富的功能（如更换快捷方式图标、选择系统图标），建议使用[AHK-ChangeIcon](https://github.com/iKineticate/AHK-ChangeIcon)。

Note: The Windows terminal cannot display icons. For richer functionality like changing shortcut icons, it is recommended to use [AHK-ChangeIcon](https://github.com/iKineticate/AHK-ChangeIcon).