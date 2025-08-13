## wml

### A sample minecraft command launcher

# 功能 

- [x] 原版下载
- [x] 支持mod加载器
- [x] morinth mod整合包解析
- [ ] curseforge mod整合包解析

# 使用方法

## 获取帮助
`wml help`

## 安装我的世界1.21.8
`wml install -v 1.21.8 --name whatever`

## 安装modloader
你需要自己下载modloader，启动器只负责生成启动脚本，这需要你指定一些参数
### example:
```bash
wml list

#这段会列出你所安装的版本，需要你自己维护，启动器会为所有安装的版本自动生成配置，存储载$HOME/.minecraft/launcher_profiles.json中

wml generate --game-path ~/.minecraft/versions/1.21.8/ --client NeoForge --mod-json ~/.minecraft/versions/neoforge-21.8.31/neoforge-21.8.31.json --output-path ~/.minecraft/versions/neoforge-21.8.31/

#这些命令的帮助可以使用‘wml generate --help’查看，请注意，--game-path末尾必须加上/以指定这是一个目录，否则启动器无法获取
```

## 安装整合包

整合包只是会把需要的文件下载下来，你需要自己安装一个带有mod加载器的版本来安装modpack，目前只有modrinth被支持
```bash
wml modpack --file ~/Downloads/test/modrinth.index.json --name whatever
```
# 这个只是我的第一个程序，所以不要对代码质量抱有期待
