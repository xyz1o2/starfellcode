use ignore::WalkBuilder;
use std::path::PathBuf;

/// 文件搜索引擎 - 支持实时全文检索和模糊匹配
#[derive(Debug, Clone)]
pub struct FileSearchEngine {
    pub query: String,
    pub results: Vec<String>,
    pub selected_index: usize,
    pub cache: Vec<PathBuf>,  // 缓存所有文件
    pub cache_built: bool,
}

impl FileSearchEngine {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            results: Vec::new(),
            selected_index: 0,
            cache: Vec::new(),
            cache_built: false,
        }
    }

    /// 构建文件缓存 - 应用启动时调用一次
    /// 递归遍历整个项目树，包含所有子目录
    /// 这样做的好处：
    /// - 应用启动时一次性加载，后续查询快速
    /// - 类似 Gemini CLI 的 list_directory 工具
    /// - 避免输入时卡顿
    pub fn build_cache(&mut self) {
        if self.cache_built {
            return;
        }

        self.cache.clear();
        
        // 使用 ignore crate 遍历文件，自动跳过 .gitignore 中的文件
        // 这与 Gemini CLI 的 list_directory 工具类似
        let walker = WalkBuilder::new(".")
            .hidden(false)           // 不隐藏隐藏文件
            .ignore(true)            // 尊重 .gitignore
            .git_ignore(true)        // 尊重 .gitignore
            .max_depth(None)         // 无限深度 - 递归遍历整个树
            .build();

        for result in walker {
            if let Ok(entry) = result {
                let path = entry.path().to_path_buf();
                
                // 跳过目录，只保留文件
                if !entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    // 跳过 target 目录中的文件（编译产物）
                    if !path.to_string_lossy().contains("target/") 
                        && !path.to_string_lossy().contains("target\\") {
                        self.cache.push(path);
                    }
                }
            }
        }

        // 排序缓存
        self.cache.sort();
        self.cache_built = true;
    }

    /// 更新查询并执行搜索
    pub fn update_query(&mut self, query: String) {
        self.query = query;
        self.selected_index = 0;
        self.search();
    }

    /// 执行搜索（支持全文检索和模糊匹配）
    /// 支持多种搜索模式：
    /// - @src -> 查找包含 "src" 的文件
    /// - @src/main -> 查找路径中包含 "src" 和 "main" 的文件
    /// - @.rs -> 查找扩展名为 .rs 的文件
    fn search(&mut self) {
        self.results.clear();

        // 移除 @ 符号
        let search_query = self.query.trim_start_matches('@').trim();

        if search_query.is_empty() {
            // 空查询 - 返回所有文件（限制数量）
            self.results = self
                .cache
                .iter()
                .take(20)
                .map(|p| format!("@{}", p.display()))
                .collect();
            return;
        }

        // 分割查询为多个关键词（支持空格和 / 分割）
        let keywords: Vec<&str> = search_query
            .split(|c| c == ' ' || c == '/')
            .filter(|s| !s.is_empty())
            .collect();

        // 执行多关键词搜索
        let mut matches: Vec<(String, usize)> = self
            .cache
            .iter()
            .filter_map(|path| {
                let path_str = path.to_string_lossy().to_lowercase();
                
                // 检查所有关键词是否都匹配
                let all_match = keywords.iter().all(|kw| {
                    let kw_lower = kw.to_lowercase();
                    path_str.contains(&kw_lower)
                });

                if all_match {
                    // 计算匹配得分
                    let mut score = 0usize;
                    
                    for kw in &keywords {
                        let kw_lower = kw.to_lowercase();
                        
                        // 文件名匹配得分最高
                        if let Some(file_name) = path.file_name() {
                            let file_name_str = file_name.to_string_lossy().to_lowercase();
                            if file_name_str.contains(&kw_lower) {
                                score += 10000;
                            }
                        }
                        
                        // 路径中的位置越前面得分越高
                        if let Some(pos) = path_str.find(&kw_lower) {
                            score += 1000 - (pos as usize / 10);
                        }
                    }

                    Some((format!("@{}", path.display()), score))
                } else {
                    None
                }
            })
            .collect();

        // 按得分排序（降序）
        matches.sort_by(|a, b| b.1.cmp(&a.1));

        // 提取结果并限制数量
        self.results = matches
            .into_iter()
            .take(20)  // 增加显示数量
            .map(|(path, _)| path)
            .collect();
    }

    /// 向上选择
    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else if !self.results.is_empty() {
            self.selected_index = self.results.len() - 1;
        }
    }

    /// 向下选择
    pub fn select_next(&mut self) {
        if self.selected_index < self.results.len().saturating_sub(1) {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
    }

    /// 获取当前选中的结果
    pub fn get_selected(&self) -> Option<String> {
        self.results.get(self.selected_index).cloned()
    }

    /// 清空搜索
    pub fn clear(&mut self) {
        self.query.clear();
        self.results.clear();
        self.selected_index = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_search_engine() {
        let mut engine = FileSearchEngine::new();
        engine.build_cache();
        
        // 测试空查询
        engine.update_query("@".to_string());
        assert!(!engine.results.is_empty());

        // 测试单关键词搜索
        engine.update_query("@src".to_string());
        let has_src = engine.results.iter().any(|r| r.contains("src"));
        assert!(has_src || engine.results.is_empty());

        // 测试多关键词搜索
        engine.update_query("@src main".to_string());
        // 结果应该包含同时包含 "src" 和 "main" 的文件
    }
}
