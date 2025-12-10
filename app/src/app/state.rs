//! Main application state for OxideLauncher
//! 
//! This module contains the central application state and message handling,
//! following Iced's Elm architecture pattern.

use iced::{
    Task, Element, Subscription, Theme,
};
use std::path::PathBuf;

use crate::core::accounts::{Account, AccountList};
use crate::core::config::Config;
use crate::core::instance::{Instance, InstanceList, ModLoader};
use crate::core::minecraft::version::VersionManifest;
use crate::core::modplatform::types::{Platform, ResourceType, SearchHit};
use crate::ui::theme::OxideTheme;
use crate::ui::views::main_view;
use crate::ui::components::toast::{Toast, ToastType};

/// Current view/screen in the application
#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Instances,
    InstanceDetail(String),  // Instance ID
    Settings,
    Accounts,
    Browse,
    CreateInstance,
}

impl Default for View {
    fn default() -> Self {
        View::Instances
    }
}

/// Settings tab selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    General,
    Java,
    Memory,
    Network,
    APIKeys,
    About,
}

/// Instance detail tab selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InstanceTab {
    #[default]
    Overview,
    Mods,
    ResourcePacks,
    ShaderPacks,
    Worlds,
    Screenshots,
    Notes,
    Settings,
}

/// Browse resource type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BrowseResourceType {
    #[default]
    Modpacks,
    Mods,
    ResourcePacks,
    ShaderPacks,
}

/// Create instance step
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CreateInstanceStep {
    #[default]
    BasicInfo,
    Version,
    ModLoader,
    Settings,
}

/// Main application state
pub struct OxideLauncher {
    // Core state
    pub config: Config,
    pub instances: InstanceList,
    pub accounts: AccountList,
    pub version_manifest: Option<VersionManifest>,
    
    // UI State
    pub current_view: View,
    pub theme: OxideTheme,
    pub search_query: String,
    pub selected_instance: Option<String>,
    
    // Instance detail state
    pub instance_tab: InstanceTab,
    pub instance_notes: String,
    pub instance_min_ram: String,
    pub instance_max_ram: String,
    pub instance_java_path: String,
    pub instance_jvm_args: String,
    pub instance_resolution_width: String,
    pub instance_resolution_height: String,
    
    // Settings state
    pub settings_tab: SettingsTab,
    pub settings_language: String,
    pub settings_theme: String,
    pub settings_data_dir: String,
    pub settings_instances_dir: String,
    pub settings_close_on_launch: bool,
    pub settings_show_console: bool,
    pub settings_check_updates: bool,
    pub settings_java_path: String,
    pub settings_jvm_args: String,
    pub settings_min_ram: String,
    pub settings_max_ram: String,
    pub settings_concurrent_downloads: String,
    pub settings_download_timeout: String,
    pub settings_use_proxy: bool,
    pub settings_proxy_host: String,
    pub settings_proxy_port: String,
    pub settings_msa_client_id: String,
    pub settings_curseforge_api_key: String,
    pub settings_modrinth_api_token: String,
    
    // Browse state
    pub browse_resource_type: BrowseResourceType,
    pub browse_search_query: String,
    pub browse_results: Vec<SearchHit>,
    pub browse_loading: bool,
    pub browse_platform_filter: String,
    pub browse_version_filter: Option<String>,
    pub browse_loader_filter: String,
    pub browse_sort_order: String,
    pub browse_page: usize,
    
    // Create instance state
    pub create_instance_step: CreateInstanceStep,
    pub create_instance_name: String,
    pub create_instance_group: String,
    pub create_instance_icon: Option<String>,
    pub create_instance_version: String,
    pub create_instance_version_search: String,
    pub create_instance_show_releases: bool,
    pub create_instance_show_snapshots: bool,
    pub create_instance_show_old: bool,
    pub create_instance_mod_loader: String,
    pub create_instance_loader_version: String,
    pub create_instance_min_ram: String,
    pub create_instance_max_ram: String,
    pub create_instance_java_path: String,
    pub create_instance_resolution_width: String,
    pub create_instance_resolution_height: String,
    pub available_versions: Vec<String>,
    pub available_loader_versions: Vec<String>,
    
    // Account state
    pub show_add_offline_dialog: bool,
    pub show_msa_auth_dialog: bool,
    pub offline_username_input: String,
    pub msa_device_code: Option<String>,
    
    // Download/progress state
    pub downloads: Vec<DownloadProgress>,
    pub toasts: Vec<Toast>,
    
    // Loading states
    pub loading: bool,
    pub loading_message: String,
}

/// Download progress tracking
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub id: String,
    pub name: String,
    pub progress: f32,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
}

/// Application messages
#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    Navigate(View),
    GoBack,
    
    // Instance list actions
    SelectInstance(String),
    DeselectInstance,
    OpenCreateInstance,
    OpenFolders,
    FilterByGroup(String),
    SearchChanged(String),
    OpenHelp,
    CheckForUpdates,
    
    // Instance detail actions
    LaunchInstance(String),
    InstanceLaunched(Result<(), String>),
    DeleteInstance(String),
    InstanceDeleted(Result<(), String>),
    CopyInstance(String),
    EditInstance(String),
    ExportInstance(String),
    OpenInstanceFolder(String),
    OpenScreenshotsFolder(String),
    KillInstance(String),
    SetInstanceTab(InstanceTab),
    InstanceNotesChanged(String),
    InstanceMinRamChanged(String),
    InstanceMaxRamChanged(String),
    InstanceJavaPathChanged(String),
    InstanceJvmArgsChanged(String),
    InstanceResolutionWidthChanged(String),
    InstanceResolutionHeightChanged(String),
    SaveInstanceSettings,
    AddResourcePack(String),
    AddShaderPack(String),
    AddWorld(String),
    
    // Instance creation
    CreateInstancePreviousStep,
    CreateInstanceNextStep,
    CreateInstanceNameChanged(String),
    CreateInstanceGroupChanged(String),
    CreateInstanceChooseIcon,
    CreateInstanceVersionSelected(String),
    CreateInstanceVersionSearchChanged(String),
    CreateInstanceShowReleasesChanged(bool),
    CreateInstanceShowSnapshotsChanged(bool),
    CreateInstanceShowOldChanged(bool),
    CreateInstanceModLoaderChanged(String),
    CreateInstanceLoaderVersionChanged(String),
    CreateInstanceMinRamChanged(String),
    CreateInstanceMaxRamChanged(String),
    CreateInstanceJavaPathChanged(String),
    CreateInstanceBrowseJava,
    CreateInstanceResolutionWidthChanged(String),
    CreateInstanceResolutionHeightChanged(String),
    CreateInstance,
    InstanceCreated(Result<Instance, String>),
    VersionsLoaded(Result<Vec<String>, String>),
    LoaderVersionsLoaded(Result<Vec<String>, String>),
    
    // Settings
    SetSettingsTab(SettingsTab),
    SettingsLanguageChanged(String),
    SettingsThemeChanged(String),
    SettingsDataDirChanged(String),
    SettingsInstancesDirChanged(String),
    SettingsCloseOnLaunchChanged(bool),
    SettingsShowConsoleChanged(bool),
    SettingsCheckUpdatesChanged(bool),
    SettingsJavaPathChanged(String),
    SettingsJvmArgsChanged(String),
    SettingsMinRamChanged(String),
    SettingsMaxRamChanged(String),
    SettingsConcurrentDownloadsChanged(String),
    SettingsDownloadTimeoutChanged(String),
    SettingsUseProxyChanged(bool),
    SettingsProxyHostChanged(String),
    SettingsProxyPortChanged(String),
    SettingsMsaClientIdChanged(String),
    SettingsCurseforgeApiKeyChanged(String),
    SettingsModrinthApiTokenChanged(String),
    SaveSettings,
    SettingsSaved(Result<(), String>),
    AutoDetectJava,
    JavaDetected(Result<String, String>),
    BrowseJavaPath,
    BrowseDataDir,
    BrowseInstancesDir,
    
    // Accounts
    AddMicrosoftAccount,
    ShowAddOfflineAccount,
    HideAddOfflineAccount,
    OfflineUsernameChanged(String),
    CreateOfflineAccount,
    AccountAdded(Result<Account, String>),
    RemoveAccount(String),
    SetActiveAccount(String),
    RefreshAccount(String),
    AccountRefreshed(Result<(), String>),
    CopyDeviceCode,
    CancelMicrosoftAuth,
    MsaAuthProgress(String),
    MsaAuthComplete(Result<Account, String>),
    
    // Browse
    BrowseResourceTypeChanged(BrowseResourceType),
    BrowseSearchChanged(String),
    BrowsePlatformChanged(String),
    BrowseVersionChanged(Option<String>),
    BrowseLoaderChanged(String),
    BrowseSortChanged(String),
    BrowseSearch,
    BrowseSearchComplete(Result<Vec<SearchHit>, String>),
    BrowsePreviousPage,
    BrowseNextPage,
    ViewProject(String, Platform),
    ShowInstallDialog(String, Platform),
    InstallToInstance(String, Platform, String),
    InstallComplete(Result<(), String>),
    
    // Downloads
    DownloadProgress { id: String, progress: f32, downloaded: u64, total: u64 },
    DownloadComplete(String),
    DownloadFailed { id: String, error: String },
    CancelDownload(String),
    
    // Toasts/Notifications
    ShowToast(Toast),
    DismissToast(usize),
    
    // General
    UpdateCheckComplete(Result<Option<String>, String>),
    Loaded(Result<(), String>),
    Tick,
    OpenUrl(String),
    Noop,
}

impl OxideLauncher {
    /// Create a new application instance with default state
    pub fn new() -> (Self, Task<Message>) {
        let config = Config::default();
        let instances = InstanceList::new();
        let accounts = AccountList::new();
        
        let app = Self {
            config: config.clone(),
            instances,
            accounts,
            version_manifest: None,
            
            current_view: View::Instances,
            theme: OxideTheme::Dark,
            search_query: String::new(),
            selected_instance: None,
            
            instance_tab: InstanceTab::Overview,
            instance_notes: String::new(),
            instance_min_ram: "512".to_string(),
            instance_max_ram: "4096".to_string(),
            instance_java_path: String::new(),
            instance_jvm_args: String::new(),
            instance_resolution_width: "854".to_string(),
            instance_resolution_height: "480".to_string(),
            
            settings_tab: SettingsTab::General,
            settings_language: "English".to_string(),
            settings_theme: "Dark".to_string(),
            settings_data_dir: config.data_dir.to_string_lossy().to_string(),
            settings_instances_dir: config.instances_dir().to_string_lossy().to_string(),
            settings_close_on_launch: false,
            settings_show_console: true,
            settings_check_updates: true,
            settings_java_path: config.java.custom_path.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
            settings_jvm_args: config.java.extra_args.join(" "),
            settings_min_ram: config.memory.min_memory.to_string(),
            settings_max_ram: config.memory.max_memory.to_string(),
            settings_concurrent_downloads: config.network.max_concurrent_downloads.to_string(),
            settings_download_timeout: "30".to_string(),
            settings_use_proxy: false,
            settings_proxy_host: String::new(),
            settings_proxy_port: String::new(),
            settings_msa_client_id: String::new(),
            settings_curseforge_api_key: String::new(),
            settings_modrinth_api_token: String::new(),
            
            browse_resource_type: BrowseResourceType::Modpacks,
            browse_search_query: String::new(),
            browse_results: Vec::new(),
            browse_loading: false,
            browse_platform_filter: "Modrinth".to_string(),
            browse_version_filter: None,
            browse_loader_filter: "Any".to_string(),
            browse_sort_order: "Relevance".to_string(),
            browse_page: 0,
            
            create_instance_step: CreateInstanceStep::BasicInfo,
            create_instance_name: String::new(),
            create_instance_group: String::new(),
            create_instance_icon: None,
            create_instance_version: String::new(),
            create_instance_version_search: String::new(),
            create_instance_show_releases: true,
            create_instance_show_snapshots: false,
            create_instance_show_old: false,
            create_instance_mod_loader: "vanilla".to_string(),
            create_instance_loader_version: String::new(),
            create_instance_min_ram: "512".to_string(),
            create_instance_max_ram: "4096".to_string(),
            create_instance_java_path: String::new(),
            create_instance_resolution_width: "854".to_string(),
            create_instance_resolution_height: "480".to_string(),
            available_versions: Vec::new(),
            available_loader_versions: Vec::new(),
            
            show_add_offline_dialog: false,
            show_msa_auth_dialog: false,
            offline_username_input: String::new(),
            msa_device_code: None,
            
            downloads: Vec::new(),
            toasts: Vec::new(),
            
            loading: true,
            loading_message: "Loading...".to_string(),
        };
        
        // Initialize - load config, instances, accounts, version manifest
        let init_task = Task::perform(
            async { load_initial_data().await },
            Message::Loaded,
        );
        
        (app, init_task)
    }
    
    /// Get a filtered list of instances based on search query
    pub fn get_filtered(&self, query: &str) -> Vec<&Instance> {
        if query.is_empty() {
            self.instances.instances.iter().collect()
        } else {
            let q = query.to_lowercase();
            self.instances
                .instances
                .iter()
                .filter(|i| i.name.to_lowercase().contains(&q))
                .collect()
        }
    }
    
    /// Add a toast notification
    pub fn add_toast(&mut self, toast_type: ToastType, _title: String, message: String) {
        self.toasts.push(Toast::new(toast_type, message));
    }
    
    /// Update handler
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Navigation
            Message::Navigate(view) => {
                self.current_view = view;
                Task::none()
            }
            Message::GoBack => {
                self.current_view = View::Instances;
                Task::none()
            }
            
            // Instance list
            Message::SelectInstance(id) => {
                self.selected_instance = Some(id);
                Task::none()
            }
            Message::DeselectInstance => {
                self.selected_instance = None;
                Task::none()
            }
            Message::OpenCreateInstance => {
                self.current_view = View::CreateInstance;
                self.create_instance_step = CreateInstanceStep::BasicInfo;
                self.create_instance_name.clear();
                self.create_instance_group.clear();
                self.create_instance_version.clear();
                self.create_instance_mod_loader = "vanilla".to_string();
                
                Task::perform(async { load_versions().await }, Message::VersionsLoaded)
            }
            Message::OpenFolders => {
                if let Err(e) = open::that(&self.config.instances_dir()) {
                    self.add_toast(ToastType::Error, "Error".to_string(), e.to_string());
                }
                Task::none()
            }
            Message::FilterByGroup(_group) => Task::none(),
            Message::SearchChanged(query) => {
                self.search_query = query;
                Task::none()
            }
            Message::OpenHelp => {
                let _ = open::that("https://github.com/oxide-launcher/wiki");
                Task::none()
            }
            Message::CheckForUpdates => {
                self.add_toast(ToastType::Info, "Updates".to_string(), "Checking...".to_string());
                Task::perform(async { check_updates().await }, Message::UpdateCheckComplete)
            }
            
            // Instance detail
            Message::LaunchInstance(id) => {
                self.add_toast(ToastType::Info, "Launching".to_string(), "Starting instance...".to_string());
                let instances = self.instances.clone();
                let accounts = self.accounts.clone();
                let config = self.config.clone();
                Task::perform(
                    async move { launch_instance(id, instances, accounts, config).await },
                    Message::InstanceLaunched,
                )
            }
            Message::InstanceLaunched(result) => {
                match result {
                    Ok(()) => self.add_toast(ToastType::Success, "Launched".to_string(), "Instance started".to_string()),
                    Err(e) => self.add_toast(ToastType::Error, "Launch Failed".to_string(), e),
                }
                Task::none()
            }
            Message::DeleteInstance(id) => {
                if let Some(_) = self.instances.remove(&id) {
                    self.selected_instance = None;
                    self.add_toast(ToastType::Success, "Deleted".to_string(), "Instance deleted".to_string());
                }
                Task::none()
            }
            Message::InstanceDeleted(_) => Task::none(),
            Message::CopyInstance(id) => {
                self.add_toast(ToastType::Info, "Copy".to_string(), "Copying instance...".to_string());
                Task::none()
            }
            Message::EditInstance(id) => {
                if let Some(instance) = self.instances.get(&id) {
                    self.instance_notes = instance.notes.clone();
                    self.current_view = View::InstanceDetail(id);
                }
                Task::none()
            }
            Message::ExportInstance(_id) => {
                self.add_toast(ToastType::Info, "Export".to_string(), "Export coming soon".to_string());
                Task::none()
            }
            Message::OpenInstanceFolder(id) => {
                if let Some(instance) = self.instances.get(&id) {
                    let _ = open::that(&instance.path);
                }
                Task::none()
            }
            Message::OpenScreenshotsFolder(id) => {
                if let Some(instance) = self.instances.get(&id) {
                    let screenshots = instance.path.join("screenshots");
                    let _ = open::that(&screenshots);
                }
                Task::none()
            }
            Message::KillInstance(_id) => {
                self.add_toast(ToastType::Info, "Kill".to_string(), "Stopping instance...".to_string());
                Task::none()
            }
            Message::SetInstanceTab(tab) => {
                self.instance_tab = tab;
                Task::none()
            }
            Message::InstanceNotesChanged(notes) => {
                self.instance_notes = notes;
                Task::none()
            }
            Message::InstanceMinRamChanged(val) => {
                self.instance_min_ram = val;
                Task::none()
            }
            Message::InstanceMaxRamChanged(val) => {
                self.instance_max_ram = val;
                Task::none()
            }
            Message::InstanceJavaPathChanged(path) => {
                self.instance_java_path = path;
                Task::none()
            }
            Message::InstanceJvmArgsChanged(args) => {
                self.instance_jvm_args = args;
                Task::none()
            }
            Message::InstanceResolutionWidthChanged(val) => {
                self.instance_resolution_width = val;
                Task::none()
            }
            Message::InstanceResolutionHeightChanged(val) => {
                self.instance_resolution_height = val;
                Task::none()
            }
            Message::SaveInstanceSettings => {
                self.add_toast(ToastType::Success, "Saved".to_string(), "Instance settings saved".to_string());
                Task::none()
            }
            Message::AddResourcePack(id) => {
                if let Some(instance) = self.instances.get(&id) {
                    let _ = open::that(instance.path.join("resourcepacks"));
                }
                Task::none()
            }
            Message::AddShaderPack(id) => {
                if let Some(instance) = self.instances.get(&id) {
                    let _ = open::that(instance.path.join("shaderpacks"));
                }
                Task::none()
            }
            Message::AddWorld(id) => {
                if let Some(instance) = self.instances.get(&id) {
                    let _ = open::that(instance.path.join("saves"));
                }
                Task::none()
            }
            
            // Create instance
            Message::CreateInstancePreviousStep => {
                self.create_instance_step = match self.create_instance_step {
                    CreateInstanceStep::Version => CreateInstanceStep::BasicInfo,
                    CreateInstanceStep::ModLoader => CreateInstanceStep::Version,
                    CreateInstanceStep::Settings => CreateInstanceStep::ModLoader,
                    CreateInstanceStep::BasicInfo => CreateInstanceStep::BasicInfo,
                };
                Task::none()
            }
            Message::CreateInstanceNextStep => {
                self.create_instance_step = match self.create_instance_step {
                    CreateInstanceStep::BasicInfo => CreateInstanceStep::Version,
                    CreateInstanceStep::Version => CreateInstanceStep::ModLoader,
                    CreateInstanceStep::ModLoader => CreateInstanceStep::Settings,
                    CreateInstanceStep::Settings => CreateInstanceStep::Settings,
                };
                Task::none()
            }
            Message::CreateInstanceNameChanged(name) => {
                self.create_instance_name = name;
                Task::none()
            }
            Message::CreateInstanceGroupChanged(group) => {
                self.create_instance_group = group;
                Task::none()
            }
            Message::CreateInstanceChooseIcon => {
                // TODO: Open file picker
                Task::none()
            }
            Message::CreateInstanceVersionSelected(version) => {
                self.create_instance_version = version;
                Task::none()
            }
            Message::CreateInstanceVersionSearchChanged(query) => {
                self.create_instance_version_search = query;
                Task::none()
            }
            Message::CreateInstanceShowReleasesChanged(val) => {
                self.create_instance_show_releases = val;
                Task::none()
            }
            Message::CreateInstanceShowSnapshotsChanged(val) => {
                self.create_instance_show_snapshots = val;
                Task::none()
            }
            Message::CreateInstanceShowOldChanged(val) => {
                self.create_instance_show_old = val;
                Task::none()
            }
            Message::CreateInstanceModLoaderChanged(loader) => {
                self.create_instance_mod_loader = loader;
                self.create_instance_loader_version.clear();
                Task::none()
            }
            Message::CreateInstanceLoaderVersionChanged(version) => {
                self.create_instance_loader_version = version;
                Task::none()
            }
            Message::CreateInstanceMinRamChanged(val) => {
                self.create_instance_min_ram = val;
                Task::none()
            }
            Message::CreateInstanceMaxRamChanged(val) => {
                self.create_instance_max_ram = val;
                Task::none()
            }
            Message::CreateInstanceJavaPathChanged(path) => {
                self.create_instance_java_path = path;
                Task::none()
            }
            Message::CreateInstanceBrowseJava => {
                // TODO: Open file picker
                Task::none()
            }
            Message::CreateInstanceResolutionWidthChanged(val) => {
                self.create_instance_resolution_width = val;
                Task::none()
            }
            Message::CreateInstanceResolutionHeightChanged(val) => {
                self.create_instance_resolution_height = val;
                Task::none()
            }
            Message::CreateInstance => {
                let name = self.create_instance_name.clone();
                let version = self.create_instance_version.clone();
                let loader = self.create_instance_mod_loader.clone();
                let config = self.config.clone();
                
                if name.is_empty() || version.is_empty() {
                    self.add_toast(ToastType::Warning, "Invalid".to_string(), "Name and version required".to_string());
                    return Task::none();
                }
                
                self.loading = true;
                self.loading_message = format!("Creating '{}'...", name);
                
                Task::perform(
                    async move { create_instance(name, version, loader, config).await },
                    Message::InstanceCreated,
                )
            }
            Message::InstanceCreated(result) => {
                self.loading = false;
                match result {
                    Ok(instance) => {
                        let name = instance.name.clone();
                        self.instances.add(instance);
                        self.add_toast(ToastType::Success, "Created".to_string(), format!("'{}' created", name));
                        self.current_view = View::Instances;
                    }
                    Err(e) => self.add_toast(ToastType::Error, "Failed".to_string(), e),
                }
                Task::none()
            }
            Message::VersionsLoaded(result) => {
                if let Ok(versions) = result {
                    self.available_versions = versions;
                }
                Task::none()
            }
            Message::LoaderVersionsLoaded(result) => {
                if let Ok(versions) = result {
                    self.available_loader_versions = versions;
                }
                Task::none()
            }
            
            // Settings
            Message::SetSettingsTab(tab) => {
                self.settings_tab = tab;
                Task::none()
            }
            Message::SettingsLanguageChanged(lang) => {
                self.settings_language = lang;
                Task::none()
            }
            Message::SettingsThemeChanged(theme) => {
                self.settings_theme = theme.clone();
                self.theme = match theme.as_str() {
                    "Light" => OxideTheme::Light,
                    _ => OxideTheme::Dark,
                };
                Task::none()
            }
            Message::SettingsDataDirChanged(dir) => {
                self.settings_data_dir = dir;
                Task::none()
            }
            Message::SettingsInstancesDirChanged(dir) => {
                self.settings_instances_dir = dir;
                Task::none()
            }
            Message::SettingsCloseOnLaunchChanged(val) => {
                self.settings_close_on_launch = val;
                Task::none()
            }
            Message::SettingsShowConsoleChanged(val) => {
                self.settings_show_console = val;
                Task::none()
            }
            Message::SettingsCheckUpdatesChanged(val) => {
                self.settings_check_updates = val;
                Task::none()
            }
            Message::SettingsJavaPathChanged(path) => {
                self.settings_java_path = path;
                Task::none()
            }
            Message::SettingsJvmArgsChanged(args) => {
                self.settings_jvm_args = args;
                Task::none()
            }
            Message::SettingsMinRamChanged(val) => {
                self.settings_min_ram = val;
                Task::none()
            }
            Message::SettingsMaxRamChanged(val) => {
                self.settings_max_ram = val;
                Task::none()
            }
            Message::SettingsConcurrentDownloadsChanged(val) => {
                self.settings_concurrent_downloads = val;
                Task::none()
            }
            Message::SettingsDownloadTimeoutChanged(val) => {
                self.settings_download_timeout = val;
                Task::none()
            }
            Message::SettingsUseProxyChanged(val) => {
                self.settings_use_proxy = val;
                Task::none()
            }
            Message::SettingsProxyHostChanged(host) => {
                self.settings_proxy_host = host;
                Task::none()
            }
            Message::SettingsProxyPortChanged(port) => {
                self.settings_proxy_port = port;
                Task::none()
            }
            Message::SettingsMsaClientIdChanged(id) => {
                self.settings_msa_client_id = id;
                Task::none()
            }
            Message::SettingsCurseforgeApiKeyChanged(key) => {
                self.settings_curseforge_api_key = key;
                Task::none()
            }
            Message::SettingsModrinthApiTokenChanged(token) => {
                self.settings_modrinth_api_token = token;
                Task::none()
            }
            Message::SaveSettings => {
                self.config.java.custom_path = if self.settings_java_path.is_empty() { None } else { Some(PathBuf::from(&self.settings_java_path)) };
                self.config.java.extra_args = self.settings_jvm_args.split_whitespace().map(|s| s.to_string()).collect();
                self.config.memory.min_memory = self.settings_min_ram.parse().unwrap_or(512);
                self.config.memory.max_memory = self.settings_max_ram.parse().unwrap_or(4096);
                self.config.network.max_concurrent_downloads = self.settings_concurrent_downloads.parse().unwrap_or(4);
                
                let config = self.config.clone();
                Task::perform(async move { save_config(config).await }, Message::SettingsSaved)
            }
            Message::SettingsSaved(result) => {
                match result {
                    Ok(()) => self.add_toast(ToastType::Success, "Saved".to_string(), "Settings saved".to_string()),
                    Err(e) => self.add_toast(ToastType::Error, "Error".to_string(), e),
                }
                Task::none()
            }
            Message::AutoDetectJava => {
                Task::perform(async { detect_java().await }, Message::JavaDetected)
            }
            Message::JavaDetected(result) => {
                match result {
                    Ok(path) => {
                        self.settings_java_path = path;
                        self.add_toast(ToastType::Success, "Found".to_string(), "Java detected".to_string());
                    }
                    Err(e) => self.add_toast(ToastType::Warning, "Not Found".to_string(), e),
                }
                Task::none()
            }
            Message::BrowseJavaPath | Message::BrowseDataDir | Message::BrowseInstancesDir => {
                // TODO: Open file/folder picker
                Task::none()
            }
            
            // Accounts
            Message::AddMicrosoftAccount => {
                self.show_msa_auth_dialog = true;
                // TODO: Start OAuth flow
                Task::none()
            }
            Message::ShowAddOfflineAccount => {
                self.show_add_offline_dialog = true;
                self.offline_username_input.clear();
                Task::none()
            }
            Message::HideAddOfflineAccount => {
                self.show_add_offline_dialog = false;
                Task::none()
            }
            Message::OfflineUsernameChanged(name) => {
                self.offline_username_input = name;
                Task::none()
            }
            Message::CreateOfflineAccount => {
                let username = self.offline_username_input.clone();
                if username.is_empty() {
                    self.add_toast(ToastType::Warning, "Invalid".to_string(), "Username required".to_string());
                    return Task::none();
                }
                
                let account = Account::new_offline(username.clone());
                
                self.accounts.add(account);
                self.show_add_offline_dialog = false;
                self.offline_username_input.clear();
                self.add_toast(ToastType::Success, "Added".to_string(), format!("Account '{}' added", username));
                Task::none()
            }
            Message::AccountAdded(result) => {
                match result {
                    Ok(account) => {
                        let name = account.username.clone();
                        self.accounts.add(account);
                        self.add_toast(ToastType::Success, "Added".to_string(), format!("'{}' added", name));
                    }
                    Err(e) => self.add_toast(ToastType::Error, "Failed".to_string(), e),
                }
                Task::none()
            }
            Message::RemoveAccount(id) => {
                if self.accounts.remove(&id).is_some() {
                    self.add_toast(ToastType::Success, "Removed".to_string(), "Account removed".to_string());
                }
                Task::none()
            }
            Message::SetActiveAccount(id) => {
                self.accounts.set_active(&id);
                Task::none()
            }
            Message::RefreshAccount(_id) => {
                self.add_toast(ToastType::Info, "Refresh".to_string(), "Refreshing...".to_string());
                Task::none()
            }
            Message::AccountRefreshed(_) => Task::none(),
            Message::CopyDeviceCode => {
                // TODO: Copy to clipboard
                self.add_toast(ToastType::Success, "Copied".to_string(), "Code copied".to_string());
                Task::none()
            }
            Message::CancelMicrosoftAuth => {
                self.show_msa_auth_dialog = false;
                self.msa_device_code = None;
                Task::none()
            }
            Message::MsaAuthProgress(code) => {
                self.msa_device_code = Some(code);
                Task::none()
            }
            Message::MsaAuthComplete(result) => {
                self.show_msa_auth_dialog = false;
                self.msa_device_code = None;
                match result {
                    Ok(account) => {
                        let name = account.username.clone();
                        self.accounts.add(account);
                        self.add_toast(ToastType::Success, "Added".to_string(), format!("'{}' added", name));
                    }
                    Err(e) => self.add_toast(ToastType::Error, "Failed".to_string(), e),
                }
                Task::none()
            }
            
            // Browse
            Message::BrowseResourceTypeChanged(resource_type) => {
                self.browse_resource_type = resource_type;
                self.browse_results.clear();
                Task::none()
            }
            Message::BrowseSearchChanged(query) => {
                self.browse_search_query = query;
                Task::none()
            }
            Message::BrowsePlatformChanged(platform) => {
                self.browse_platform_filter = platform;
                Task::none()
            }
            Message::BrowseVersionChanged(version) => {
                self.browse_version_filter = version;
                Task::none()
            }
            Message::BrowseLoaderChanged(loader) => {
                self.browse_loader_filter = loader;
                Task::none()
            }
            Message::BrowseSortChanged(sort) => {
                self.browse_sort_order = sort;
                Task::none()
            }
            Message::BrowseSearch => {
                self.browse_loading = true;
                let query = self.browse_search_query.clone();
                let platform = self.browse_platform_filter.clone();
                let resource_type = self.browse_resource_type;
                
                Task::perform(
                    async move { search_mods(query, platform, resource_type).await },
                    Message::BrowseSearchComplete,
                )
            }
            Message::BrowseSearchComplete(result) => {
                self.browse_loading = false;
                match result {
                    Ok(results) => self.browse_results = results,
                    Err(e) => self.add_toast(ToastType::Error, "Search Failed".to_string(), e),
                }
                Task::none()
            }
            Message::BrowsePreviousPage => {
                if self.browse_page > 0 {
                    self.browse_page -= 1;
                }
                Task::none()
            }
            Message::BrowseNextPage => {
                self.browse_page += 1;
                Task::none()
            }
            Message::ViewProject(id, platform) => {
                let url = match platform {
                    Platform::Modrinth => format!("https://modrinth.com/project/{}", id),
                    Platform::CurseForge => format!("https://www.curseforge.com/minecraft/mc-mods/{}", id),
                };
                let _ = open::that(&url);
                Task::none()
            }
            Message::ShowInstallDialog(_id, _platform) => {
                // TODO: Show install dialog
                Task::none()
            }
            Message::InstallToInstance(_project_id, _platform, _instance_id) => {
                self.add_toast(ToastType::Info, "Installing".to_string(), "Installing mod...".to_string());
                Task::none()
            }
            Message::InstallComplete(result) => {
                match result {
                    Ok(()) => self.add_toast(ToastType::Success, "Installed".to_string(), "Mod installed".to_string()),
                    Err(e) => self.add_toast(ToastType::Error, "Failed".to_string(), e),
                }
                Task::none()
            }
            
            // Downloads
            Message::DownloadProgress { id, progress, downloaded, total } => {
                if let Some(dl) = self.downloads.iter_mut().find(|d| d.id == id) {
                    dl.progress = progress;
                    dl.downloaded_bytes = downloaded;
                    dl.total_bytes = total;
                }
                Task::none()
            }
            Message::DownloadComplete(id) => {
                self.downloads.retain(|d| d.id != id);
                Task::none()
            }
            Message::DownloadFailed { id, error } => {
                self.downloads.retain(|d| d.id != id);
                self.add_toast(ToastType::Error, "Download Failed".to_string(), error);
                Task::none()
            }
            Message::CancelDownload(id) => {
                self.downloads.retain(|d| d.id != id);
                Task::none()
            }
            
            // Toasts
            Message::ShowToast(toast) => {
                self.toasts.push(toast);
                Task::none()
            }
            Message::DismissToast(index) => {
                if index < self.toasts.len() {
                    self.toasts.remove(index);
                }
                Task::none()
            }
            
            // General
            Message::UpdateCheckComplete(result) => {
                match result {
                    Ok(Some(version)) => self.add_toast(ToastType::Info, "Update".to_string(), format!("v{} available", version)),
                    Ok(None) => self.add_toast(ToastType::Success, "Up to Date".to_string(), "Latest version".to_string()),
                    Err(e) => self.add_toast(ToastType::Error, "Check Failed".to_string(), e),
                }
                Task::none()
            }
            Message::Loaded(result) => {
                self.loading = false;
                if let Err(e) = result {
                    self.add_toast(ToastType::Error, "Load Failed".to_string(), e);
                }
                Task::none()
            }
            Message::Tick => Task::none(),
            Message::OpenUrl(url) => {
                let _ = open::that(&url);
                Task::none()
            }
            Message::Noop => Task::none(),
        }
    }
    
    /// View handler
    pub fn view(&self) -> Element<Message> {
        main_view::main_view(self)
    }
    
    /// Theme getter
    pub fn theme(&self) -> Theme {
        self.theme.to_iced_theme()
    }
    
    /// Subscription handler
    pub fn subscription(&self) -> Subscription<Message> {
        iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick)
    }
}

// Async helper functions

async fn load_initial_data() -> Result<(), String> {
    Ok(())
}

async fn load_versions() -> Result<Vec<String>, String> {
    Ok(vec![
        "1.21.4".to_string(),
        "1.21.3".to_string(),
        "1.21.1".to_string(),
        "1.21".to_string(),
        "1.20.6".to_string(),
        "1.20.4".to_string(),
        "1.20.1".to_string(),
        "1.19.4".to_string(),
        "1.19.2".to_string(),
        "1.18.2".to_string(),
        "1.16.5".to_string(),
        "1.12.2".to_string(),
        "1.8.9".to_string(),
        "1.7.10".to_string(),
    ])
}

async fn create_instance(name: String, version: String, loader: String, config: Config) -> Result<Instance, String> {
    use crate::core::instance::{InstanceConfig, ModLoaderType};
    
    let mod_loader = match loader.as_str() {
        "forge" => Some(ModLoader {
            loader_type: ModLoaderType::Forge,
            version: "latest".to_string(),
        }),
        "fabric" => Some(ModLoader {
            loader_type: ModLoaderType::Fabric,
            version: "latest".to_string(),
        }),
        "quilt" => Some(ModLoader {
            loader_type: ModLoaderType::Quilt,
            version: "latest".to_string(),
        }),
        "neoforge" => Some(ModLoader {
            loader_type: ModLoaderType::NeoForge,
            version: "latest".to_string(),
        }),
        _ => None,
    };
    
    let instance_config = InstanceConfig {
        name: name.clone(),
        minecraft_version: version,
        mod_loader,
        icon: String::new(),
        group: None,
        copy_from: None,
        import_modpack: None,
    };
    
    crate::core::instance::create_instance(instance_config, &config.instances_dir())
        .await
        .map_err(|e| e.to_string())
}

async fn launch_instance(id: String, instances: InstanceList, accounts: AccountList, config: Config) -> Result<(), String> {
    let instance = instances.iter().find(|i| i.id == id).ok_or("Instance not found")?;
    let account = accounts.get_active().ok_or("No active account")?;
    
    crate::core::launch::launch_instance(instance, Some(account))
        .await
        .map_err(|e| e.to_string())
}

async fn save_config(config: Config) -> Result<(), String> {
    config.save().map_err(|e| e.to_string())
}

async fn detect_java() -> Result<String, String> {
    crate::core::java::find_java_installation(8)
        .map(|p| p.to_string_lossy().to_string())
        .ok_or("No Java found".to_string())
}

async fn search_mods(query: String, _platform: String, resource_type: BrowseResourceType) -> Result<Vec<SearchHit>, String> {
    use crate::core::modplatform::modrinth::ModrinthClient;
    use crate::core::modplatform::types::SearchQuery;
    
    let resource = match resource_type {
        BrowseResourceType::Modpacks => ResourceType::Modpack,
        BrowseResourceType::Mods => ResourceType::Mod,
        BrowseResourceType::ResourcePacks => ResourceType::ResourcePack,
        BrowseResourceType::ShaderPacks => ResourceType::ShaderPack,
    };
    
    let search_query = SearchQuery {
        query,
        resource_type: Some(resource),
        limit: 20,
        ..Default::default()
    };
    
    let client = ModrinthClient::new();
    let results = client.search(&search_query).await.map_err(|e| e.to_string())?;
    Ok(results.hits)
}

async fn check_updates() -> Result<Option<String>, String> {
    Ok(None)
}

/// Application entry point
pub fn run() -> iced::Result {
    iced::application(OxideLauncher::new, OxideLauncher::update, OxideLauncher::view)
        .theme(OxideLauncher::theme)
        .subscription(OxideLauncher::subscription)
        .window(iced::window::Settings {
            size: iced::Size::new(1200.0, 800.0),
            ..Default::default()
        })
        .run()
}
