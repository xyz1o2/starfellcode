/// 高效的 Ratatui 渲染缓存系统
/// 
/// 核心特性：
/// - 缓存已渲染的行数据
/// - 脏标记追踪
/// - 增量渲染支持
/// - 内存池优化

use ratatui::text::Line;
use std::collections::HashMap;

/// 渲染缓存键 - 用于标识不同的渲染内容
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheKey {
    Header,
    ChatHistory,
    InputStream,
    CommandHints,
    MentionSuggestions,
}

/// 脏标记 - 追踪哪些内容需要重新渲染
#[derive(Debug, Clone, Copy)]
pub struct DirtyFlags {
    pub header: bool,
    pub chat_history: bool,
    pub input_stream: bool,
    pub command_hints: bool,
    pub mention_suggestions: bool,
}

impl DirtyFlags {
    pub fn new() -> Self {
        Self {
            header: true,
            chat_history: true,
            input_stream: true,
            command_hints: true,
            mention_suggestions: true,
        }
    }

    pub fn mark_all_clean(&mut self) {
        self.header = false;
        self.chat_history = false;
        self.input_stream = false;
        self.command_hints = false;
        self.mention_suggestions = false;
    }

    pub fn mark_all_dirty(&mut self) {
        self.header = true;
        self.chat_history = true;
        self.input_stream = true;
        self.command_hints = true;
        self.mention_suggestions = true;
    }
}

/// 缓存的渲染数据
#[derive(Debug, Clone)]
pub struct CachedRenderData {
    pub lines: Vec<Line<'static>>,
    pub total_lines: usize,
    pub visible_lines: usize,
    pub max_scroll: usize,
    pub hash: u64,  // 用于检测内容变化
}

impl CachedRenderData {
    pub fn new(lines: Vec<Line<'static>>) -> Self {
        let total_lines = lines.len();
        Self {
            lines,
            total_lines,
            visible_lines: 0,
            max_scroll: 0,
            hash: 0,
        }
    }

    pub fn with_scroll_info(mut self, visible_lines: usize) -> Self {
        self.visible_lines = visible_lines;
        self.max_scroll = self.total_lines.saturating_sub(visible_lines);
        self
    }
}

/// 高效的渲染缓存管理器
pub struct RenderCache {
    cache: HashMap<CacheKey, CachedRenderData>,
    dirty_flags: DirtyFlags,
    
    // 性能统计
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl RenderCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            dirty_flags: DirtyFlags::new(),
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    /// 获取缓存数据（如果存在且未标记为脏）
    pub fn get(&mut self, key: CacheKey) -> Option<&CachedRenderData> {
        if self.is_dirty(key) {
            self.cache_misses += 1;
            return None;
        }

        if let Some(data) = self.cache.get(&key) {
            self.cache_hits += 1;
            return Some(data);
        }

        self.cache_misses += 1;
        None
    }

    /// 存储缓存数据
    pub fn set(&mut self, key: CacheKey, data: CachedRenderData) {
        self.cache.insert(key, data);
        self.mark_clean(key);
    }

    /// 检查是否为脏
    pub fn is_dirty(&self, key: CacheKey) -> bool {
        match key {
            CacheKey::Header => self.dirty_flags.header,
            CacheKey::ChatHistory => self.dirty_flags.chat_history,
            CacheKey::InputStream => self.dirty_flags.input_stream,
            CacheKey::CommandHints => self.dirty_flags.command_hints,
            CacheKey::MentionSuggestions => self.dirty_flags.mention_suggestions,
        }
    }

    /// 标记为脏
    pub fn mark_dirty(&mut self, key: CacheKey) {
        match key {
            CacheKey::Header => self.dirty_flags.header = true,
            CacheKey::ChatHistory => self.dirty_flags.chat_history = true,
            CacheKey::InputStream => self.dirty_flags.input_stream = true,
            CacheKey::CommandHints => self.dirty_flags.command_hints = true,
            CacheKey::MentionSuggestions => self.dirty_flags.mention_suggestions = true,
        }
    }

    /// 标记为干净
    pub fn mark_clean(&mut self, key: CacheKey) {
        match key {
            CacheKey::Header => self.dirty_flags.header = false,
            CacheKey::ChatHistory => self.dirty_flags.chat_history = false,
            CacheKey::InputStream => self.dirty_flags.input_stream = false,
            CacheKey::CommandHints => self.dirty_flags.command_hints = false,
            CacheKey::MentionSuggestions => self.dirty_flags.mention_suggestions = false,
        }
    }

    /// 清空所有缓存
    pub fn clear(&mut self) {
        self.cache.clear();
        self.dirty_flags.mark_all_dirty();
    }

    /// 获取缓存统计信息
    pub fn get_stats(&self) -> CacheStats {
        let total = self.cache_hits + self.cache_misses;
        let hit_rate = if total > 0 {
            (self.cache_hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        CacheStats {
            hits: self.cache_hits,
            misses: self.cache_misses,
            total,
            hit_rate,
            cached_items: self.cache.len(),
        }
    }

    /// 重置统计信息
    pub fn reset_stats(&mut self) {
        self.cache_hits = 0;
        self.cache_misses = 0;
    }
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub total: u64,
    pub hit_rate: f64,
    pub cached_items: usize,
}

/// 快速哈希函数 - 用于检测内容变化
pub fn quick_hash(data: &str) -> u64 {
    let mut hash: u64 = 5381;
    for byte in data.bytes() {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(byte as u64);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dirty_flags() {
        let mut flags = DirtyFlags::new();
        assert!(flags.header);
        
        flags.mark_all_clean();
        assert!(!flags.header);
        
        flags.mark_all_dirty();
        assert!(flags.header);
    }

    #[test]
    fn test_render_cache() {
        let mut cache = RenderCache::new();
        
        // 初始状态应该是脏的
        assert!(cache.is_dirty(CacheKey::Header));
        
        // 设置缓存后应该是干净的
        let data = CachedRenderData::new(vec![]);
        cache.set(CacheKey::Header, data);
        assert!(!cache.is_dirty(CacheKey::Header));
        
        // 标记为脏后应该能检测到
        cache.mark_dirty(CacheKey::Header);
        assert!(cache.is_dirty(CacheKey::Header));
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = RenderCache::new();
        cache.cache_hits = 100;
        cache.cache_misses = 50;
        
        let stats = cache.get_stats();
        assert_eq!(stats.total, 150);
        assert!((stats.hit_rate - 66.66).abs() < 0.1);
    }

    #[test]
    fn test_quick_hash() {
        let hash1 = quick_hash("hello");
        let hash2 = quick_hash("hello");
        let hash3 = quick_hash("world");
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
