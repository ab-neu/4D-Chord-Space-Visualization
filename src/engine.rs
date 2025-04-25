use crate::rgba;
use kiss3d::camera::ArcBall;
use kiss3d::event::{Action, Key, WindowEvent};
use kiss3d::light::Light;
use kiss3d::nalgebra::{Point3, Translation3};
use kiss3d::scene::SceneNode;
use kiss3d::window::Window;

// Constants for animation and visualization
const POSITION_SCALE: f32 = 1000.0;
const COLOR_SCALE: f32 = 0.03; // More extreme color changes
const MOTION_SPEED: f32 = 0.125; // 125ms per keyframe (1/16th note at 120 BPM)
const GRID_SIZE: f32 = 200.0;
const GRID_CELLS: i32 = 10;

// Animation state
struct AnimationState {
    motions: Vec<[i32; 4]>,             // Voice motion vectors
    current_position: Point3<f32>,      // Current position
    target_position: Point3<f32>,       // Target position
    current_index: usize,               // Current keyframe index
    transition_progress: f32,           // Progress through current transition (0.0-1.0)
    current_hue: f32,                   // Current color hue
    target_hue: f32,                    // Target color hue
    position_history: Vec<Point3<f32>>, // Trail of past positions
    timer: f32,                         // Timer for animation
}

impl AnimationState {
    // Create a new animation state
    fn new(motions: Vec<[i32; 4]>) -> Self {
        let current_position = Point3::new(0.0, 0.0, 0.0);

        // Calculate initial target position and hue
        let first_motion = if !motions.is_empty() {
            motions[0]
        } else {
            [0, 0, 0, 0]
        };
        let target_position = Point3::new(
            first_motion[1] as f32 * POSITION_SCALE / 100.0,
            first_motion[2] as f32 * POSITION_SCALE / 100.0,
            first_motion[3] as f32 * POSITION_SCALE / 100.0,
        );

        let initial_hue = (first_motion[0] as f32 * COLOR_SCALE).abs() % 1.0;

        Self {
            motions,
            current_position,
            target_position,
            current_index: 0,
            transition_progress: 0.0,
            current_hue: initial_hue,
            target_hue: initial_hue,
            position_history: Vec::new(),
            timer: 0.0,
        }
    }

    // Update animation state
    fn update(&mut self, delta_time: f32) -> bool {
        self.timer += delta_time;

        // Update transition progress
        self.transition_progress += delta_time / MOTION_SPEED;

        // Check if we need to move to the next keyframe
        if self.transition_progress >= 1.0 {
            // Reset transition and move to next keyframe
            self.transition_progress = 0.0;
            self.current_position = self.target_position;

            // Add to trail history
            self.position_history.push(self.current_position);
            if self.position_history.len() > 100 {
                self.position_history.remove(0);
            }

            // Move to next motion index
            self.current_index += 1;

            // Check if we've reached the end
            if self.current_index >= self.motions.len() {
                // We've reached the end, stop the animation
                println!("Animation complete - reached the end of keyframes");
                return false;
            }

            self.current_hue = self.target_hue;

            // Calculate next target hue
            let motion = self.motions[self.current_index];
            let total_motion = motion[0] as f32 * COLOR_SCALE;
            self.target_hue = total_motion.abs() % 1.0;

            // Calculate next target position
            self.target_position = Point3::new(
                self.current_position.x + motion[1] as f32 * POSITION_SCALE / 100.0,
                self.current_position.y + motion[2] as f32 * POSITION_SCALE / 100.0,
                self.current_position.z + motion[3] as f32 * POSITION_SCALE / 100.0,
            );

            /*println!(
                "Keyframe {}/{}: Position: ({:.2}, {:.2}, {:.2})",
                self.current_index + 1,
                self.motions.len(),
                self.target_position.x,
                self.target_position.y,
                self.target_position.z
            );*/
        }

        // Continue animation
        true
    }

    // Get interpolated position
    fn interpolated_position(&self) -> Point3<f32> {
        Point3::new(
            self.current_position.x
                + (self.target_position.x - self.current_position.x) * self.transition_progress,
            self.current_position.y
                + (self.target_position.y - self.current_position.y) * self.transition_progress,
            self.current_position.z
                + (self.target_position.z - self.current_position.z) * self.transition_progress,
        )
    }

    // Get interpolated color
    fn interpolated_color(&self) -> (f32, f32, f32) {
        // Interpolate hue (find shortest path around color wheel)
        let mut hue_diff = self.target_hue - self.current_hue;
        if hue_diff.abs() > 0.5 {
            hue_diff = if hue_diff > 0.0 {
                hue_diff - 1.0
            } else {
                hue_diff + 1.0
            };
        }
        let interpolated_hue = (self.current_hue + hue_diff * self.transition_progress).fract();

        // Convert HSV to RGB using our rgba module
        rgba::hsv_to_rgb(interpolated_hue, 1.0, 1.0)
    }
}

// Create grid for reference
fn create_grid(window: &mut Window) -> Vec<SceneNode> {
    let mut grid_lines = Vec::new();

    // Create grid lines along X and Z axes
    for i in -GRID_CELLS..=GRID_CELLS {
        let pos = i as f32 * GRID_SIZE;

        // Create lines using cylinders
        // X-axis lines
        let mut line_x = window.add_cylinder(2.0, GRID_SIZE * GRID_CELLS as f32 * 2.0);
        line_x.set_color(0.3, 0.3, 0.4);
        line_x.set_local_translation(Translation3::new(0.0, 0.0, pos));
        line_x.set_local_rotation(kiss3d::nalgebra::UnitQuaternion::from_axis_angle(
            &kiss3d::nalgebra::Vector3::z_axis(),
            std::f32::consts::FRAC_PI_2,
        ));
        grid_lines.push(line_x);

        // Z-axis lines
        let mut line_z = window.add_cylinder(2.0, GRID_SIZE * GRID_CELLS as f32 * 2.0);
        line_z.set_color(0.3, 0.3, 0.4);
        line_z.set_local_translation(Translation3::new(pos, 0.0, 0.0));
        line_z.set_local_rotation(kiss3d::nalgebra::UnitQuaternion::from_axis_angle(
            &kiss3d::nalgebra::Vector3::x_axis(),
            std::f32::consts::FRAC_PI_2,
        ));
        grid_lines.push(line_z);
    }

    grid_lines
}

// Create trail lines to show path
fn update_trail(window: &mut Window, state: &AnimationState, trail_nodes: &mut Vec<SceneNode>) {
    // Remove old trail nodes
    for mut node in trail_nodes.drain(..) {
        window.remove_node(&mut node);
    }

    // Add new trail segments if we have history
    if state.position_history.len() > 1 {
        for i in 1..state.position_history.len() {
            let p1 = state.position_history[i - 1];
            let p2 = state.position_history[i];

            // Create thin lines instead of cylinders
            let mut line = window.add_cylinder(1.0, 1.0); // Just a placeholder that won't be visible
            line.set_visible(false); // Don't show the cylinders

            // Get points along the line
            let num_segments = 8; // Number of points to create along the line
            for j in 0..num_segments {
                let t = j as f32 / (num_segments - 1) as f32;
                let pos = Point3::new(
                    p1.x + (p2.x - p1.x) * t,
                    p1.y + (p2.y - p1.y) * t,
                    p1.z + (p2.z - p1.z) * t,
                );

                // Create a small sphere at each point
                let mut point = window.add_sphere(1.5);
                point.set_color(0.4, 0.5, 0.6);
                point.set_local_translation(Translation3::new(pos.x, pos.y, pos.z));
                trail_nodes.push(point);
            }

            trail_nodes.push(line); // Still need to add the invisible line to clean it up later
        }

        // Add segment from last history point to current position
        if let Some(last) = state.position_history.last() {
            let current_pos = state.interpolated_position();

            // Create thin line from dotted points
            let mut line = window.add_cylinder(1.0, 1.0); // Just a placeholder
            line.set_visible(false); // Don't show the cylinder

            // Get points along the line
            let num_segments = 8; // Number of points to create along the line
            for j in 0..num_segments {
                let t = j as f32 / (num_segments - 1) as f32;
                let pos = Point3::new(
                    last.x + (current_pos.x - last.x) * t,
                    last.y + (current_pos.y - last.y) * t,
                    last.z + (current_pos.z - last.z) * t,
                );

                // Create a small sphere at each point
                let mut point = window.add_sphere(1.5);
                point.set_color(0.4, 0.5, 0.6);
                point.set_local_translation(Translation3::new(pos.x, pos.y, pos.z));
                trail_nodes.push(point);
            }

            trail_nodes.push(line); // Still need to add the invisible line
        }
    }
}

// Render function
pub fn render(transformation: Vec<[i32; 4]>) {
    if transformation.is_empty() {
        println!("No transformation data to render");
        return;
    }

    // Create window
    let mut window = Window::new("MIDI Visualization - Press ESC to exit");

    // Set background color (dark blue)
    window.set_background_color(0.05, 0.05, 0.1);

    // Add a light
    window.set_light(Light::StickToCamera);

    // Create sphere
    let mut sphere = window.add_sphere(30.0);
    sphere.set_color(1.0, 0.0, 0.0); // Initial color, will be updated

    // Create grid
    let _grid = create_grid(&mut window);

    // Storage for trail nodes
    let mut trail_nodes: Vec<SceneNode> = Vec::new();

    // Initialize animation state
    let mut state = AnimationState::new(transformation);

    // Create camera
    let eye = Point3::new(0.0, 200.0, 500.0);
    let at = Point3::new(0.0, 0.0, 0.0);
    let mut camera = ArcBall::new(eye, at);

    // Animation loop
    let mut last_time = std::time::Instant::now();
    let mut running = true;

    while window.render_with_camera(&mut camera) && running {
        // Calculate delta time
        let now = std::time::Instant::now();
        let delta_time = now.duration_since(last_time).as_secs_f32();
        last_time = now;

        // Update animation state
        running = state.update(delta_time);

        // Get current position and color
        let position = state.interpolated_position();
        let (r, g, b) = state.interpolated_color();

        // Update sphere position and color
        sphere.set_local_translation(Translation3::new(position.x, position.y, position.z));
        sphere.set_color(r, g, b);

        // Update trail
        update_trail(&mut window, &state, &mut trail_nodes);

        // Check for escape key to exit
        for event in window.events().iter() {
            if let WindowEvent::Key(key, Action::Release, _) = event.value {
                if key == Key::Escape {
                    running = false;
                    break;
                }
            }
        }
    }
}
