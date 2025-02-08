mod config_input_data_builder;
mod input_data_builder;
mod max_output_lengths_builder;
mod output_data_builder;

/**
These builders are compiled to allow the user to more easily create the required inputs for their gpu task, where everything is completely type safe according to the custom types they have defined
 */
pub use config_input_data_builder::create_config_input_data_builder;
pub use input_data_builder::create_input_data_builder;
pub use max_output_lengths_builder::create_max_output_lengths_builder;
pub use output_data_builder::create_output_data_builder;
