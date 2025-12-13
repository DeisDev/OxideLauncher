//! Launch steps
//! 
//! Each step handles a specific part of the launch process.

mod check_java;
mod verify_java;
mod auto_install_java;
mod create_game_folders;
mod extract_natives;
mod pre_launch_command;
mod post_launch_command;
mod launch_game;
mod print_instance_info;

pub use check_java::CheckJavaStep;
pub use verify_java::VerifyJavaStep;
pub use auto_install_java::AutoInstallJavaStep;
pub use create_game_folders::CreateGameFoldersStep;
pub use extract_natives::ExtractNativesStep;
pub use pre_launch_command::PreLaunchCommandStep;
pub use post_launch_command::PostLaunchCommandStep;
pub use launch_game::LaunchGameStep;
pub use print_instance_info::PrintInstanceInfoStep;

use super::task::LaunchTask;
use super::LaunchContext;

/// Create a default launch task with all standard steps
pub fn create_default_launch_task(context: LaunchContext) -> LaunchTask {
    let mut task = LaunchTask::new(context.clone());
    
    // Add steps in order
    task.append_step(Box::new(PrintInstanceInfoStep::new()));
    task.append_step(Box::new(CreateGameFoldersStep::new()));
    task.append_step(Box::new(CheckJavaStep::new()));
    task.append_step(Box::new(VerifyJavaStep::new()));
    
    // Auto-install Java if enabled
    if context.config.java.auto_download {
        task.append_step(Box::new(AutoInstallJavaStep::new()));
    }
    
    task.append_step(Box::new(ExtractNativesStep::new()));
    
    // Pre-launch command if configured
    if context.instance.settings.pre_launch_command.is_some() {
        task.append_step(Box::new(PreLaunchCommandStep::new()));
    }
    
    // The main launch step
    task.append_step(Box::new(LaunchGameStep::new()));
    
    // Post-launch command if configured
    if context.instance.settings.post_exit_command.is_some() {
        task.append_step(Box::new(PostLaunchCommandStep::new()));
    }
    
    task
}
