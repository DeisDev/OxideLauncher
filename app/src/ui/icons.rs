//! SVG icons for the UI
//! 
//! These icons are based on common icon sets and are optimized for use in the launcher.

use iced::widget::svg;

/// Icon names for easy reference
pub enum Icon {
    // Navigation
    Home,
    Settings,
    Help,
    Menu,
    Close,
    Back,
    Forward,
    
    // Actions
    Add,
    Edit,
    Delete,
    Copy,
    Paste,
    Search,
    Filter,
    Sort,
    Refresh,
    Download,
    Upload,
    Play,
    Stop,
    Pause,
    
    // Instance related
    Folder,
    FolderOpen,
    Instance,
    Minecraft,
    Java,
    Mods,
    ResourcePack,
    ShaderPack,
    World,
    
    // Accounts
    User,
    Users,
    Login,
    Logout,
    
    // Platforms
    Modrinth,
    CurseForge,
    
    // Status
    Success,
    Warning,
    Error,
    Info,
    Loading,
    
    // Mod loaders
    Forge,
    Fabric,
    Quilt,
    NeoForge,
    
    // Other
    External,
    Link,
    Heart,
    Star,
    Clock,
    Calendar,
}

impl Icon {
    /// Get the SVG data for this icon
    pub fn svg_data(&self) -> &'static str {
        match self {
            // Navigation icons
            Icon::Home => include_str!("../../assets/icons/home.svg"),
            Icon::Settings => include_str!("../../assets/icons/settings.svg"),
            Icon::Help => include_str!("../../assets/icons/help.svg"),
            Icon::Menu => include_str!("../../assets/icons/menu.svg"),
            Icon::Close => include_str!("../../assets/icons/close.svg"),
            Icon::Back => include_str!("../../assets/icons/back.svg"),
            Icon::Forward => include_str!("../../assets/icons/forward.svg"),
            
            // Action icons
            Icon::Add => include_str!("../../assets/icons/add.svg"),
            Icon::Edit => include_str!("../../assets/icons/edit.svg"),
            Icon::Delete => include_str!("../../assets/icons/delete.svg"),
            Icon::Copy => include_str!("../../assets/icons/copy.svg"),
            Icon::Paste => include_str!("../../assets/icons/paste.svg"),
            Icon::Search => include_str!("../../assets/icons/search.svg"),
            Icon::Filter => include_str!("../../assets/icons/filter.svg"),
            Icon::Sort => include_str!("../../assets/icons/sort.svg"),
            Icon::Refresh => include_str!("../../assets/icons/refresh.svg"),
            Icon::Download => include_str!("../../assets/icons/download.svg"),
            Icon::Upload => include_str!("../../assets/icons/upload.svg"),
            Icon::Play => include_str!("../../assets/icons/play.svg"),
            Icon::Stop => include_str!("../../assets/icons/stop.svg"),
            Icon::Pause => include_str!("../../assets/icons/pause.svg"),
            
            // Instance icons
            Icon::Folder => include_str!("../../assets/icons/folder.svg"),
            Icon::FolderOpen => include_str!("../../assets/icons/folder-open.svg"),
            Icon::Instance => include_str!("../../assets/icons/instance.svg"),
            Icon::Minecraft => include_str!("../../assets/icons/minecraft.svg"),
            Icon::Java => include_str!("../../assets/icons/java.svg"),
            Icon::Mods => include_str!("../../assets/icons/mods.svg"),
            Icon::ResourcePack => include_str!("../../assets/icons/resource-pack.svg"),
            Icon::ShaderPack => include_str!("../../assets/icons/shader-pack.svg"),
            Icon::World => include_str!("../../assets/icons/world.svg"),
            
            // Account icons
            Icon::User => include_str!("../../assets/icons/user.svg"),
            Icon::Users => include_str!("../../assets/icons/users.svg"),
            Icon::Login => include_str!("../../assets/icons/login.svg"),
            Icon::Logout => include_str!("../../assets/icons/logout.svg"),
            
            // Platform icons
            Icon::Modrinth => include_str!("../../assets/icons/modrinth.svg"),
            Icon::CurseForge => include_str!("../../assets/icons/curseforge.svg"),
            
            // Status icons
            Icon::Success => include_str!("../../assets/icons/success.svg"),
            Icon::Warning => include_str!("../../assets/icons/warning.svg"),
            Icon::Error => include_str!("../../assets/icons/error.svg"),
            Icon::Info => include_str!("../../assets/icons/info.svg"),
            Icon::Loading => include_str!("../../assets/icons/loading.svg"),
            
            // Mod loader icons
            Icon::Forge => include_str!("../../assets/icons/forge.svg"),
            Icon::Fabric => include_str!("../../assets/icons/fabric.svg"),
            Icon::Quilt => include_str!("../../assets/icons/quilt.svg"),
            Icon::NeoForge => include_str!("../../assets/icons/neoforge.svg"),
            
            // Other icons
            Icon::External => include_str!("../../assets/icons/external.svg"),
            Icon::Link => include_str!("../../assets/icons/link.svg"),
            Icon::Heart => include_str!("../../assets/icons/heart.svg"),
            Icon::Star => include_str!("../../assets/icons/star.svg"),
            Icon::Clock => include_str!("../../assets/icons/clock.svg"),
            Icon::Calendar => include_str!("../../assets/icons/calendar.svg"),
        }
    }

    /// Get an SVG handle for this icon
    pub fn handle(&self) -> svg::Handle {
        svg::Handle::from_memory(self.svg_data().as_bytes().to_vec())
    }
}

/// Create an SVG widget for an icon
pub fn icon(icon: Icon, size: u16) -> iced::widget::Svg<'static> {
    iced::widget::svg(icon.handle())
        .width(iced::Length::Fixed(size as f32))
        .height(iced::Length::Fixed(size as f32))
}
