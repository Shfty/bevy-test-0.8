// TODO: Factor out scene archival
//       * Currently loading / unloading entire scenes instead of archiving individual entities
//         * Suboptimal, no longer necessary now archive bundles can be defined in blender
//       * Switching from InsertSceneArchive to InsertScene doesn't work as-is
//         * Scene is spawned without GLTF extras deserialized
//         * Appears to be a timing issue - scene is spawned before deserialization
//           * However, setting up system bounds hasn't fixed it,
//             and forcing a changed event doesn't cause the scene to reinstance with extras
//           * Need to investigate further; using stock scene machinery would be ideal
//
// TODO: Migrate remaining entity assemblages to Blender scenes
//       * Will need to use a tag component to identify ship, force rotation targets
//
// TODO: Blender collision geometry pipeline
//       * Need a way to tag certain models as being collision meshes,
//         generate appropriate rapier geometry
//         * Can leverage PlaneCollider mesh slicing functionality to generate 2D convex hulls on load
//       * Also need a way to output analytical collision primitives like spheres and cuboids
//       * Should support both 2D and 3D collision
//       * Investigate custom properties
//         * https://blog.hamaluik.ca/posts/dynamic-blender-properties/
//
// FIXME: Exporting tangents breaks the sprite shader
//        * Previously working due to mesh being non-triangulated
//        * Need to strip tangents as part of the ConvertMesh2d process
//
// TODO: Better material support so line / point meshes are viable
//       * Can't render with StandardMaterial due to lack of normals
//       * Need an override component to convert the automatic StandardMaterial into something else
//       * Opens the door to custom line / point use-cases
//         * Point sprites
//           * Not an API-level feature in WGPU
//         * Geometry lines
//         * GPU instancing would be ideal, but CPU mesh puppeting is probably more viable as-is
//
// FIXME: Incorrect clipping rect for rightmost panel if it contains a world inspector
//
// TODO: Formalize mesh rotation
//       * Enemies should be able to animate their mesh rotation
//       * Currently impeded by gimbal lock caused by top-level rotation + playfield rotation
//
// TODO: Enemy pooling
//       * Generalize bullet pool system
//
// TODO: Formalize bullet animation setup
//       * Currently hardcoded to spawn with two-point transform lerp,
//         update stops when unpooled
//       * Need both of these to be configurable from calling code
//         * Should be able to spawn bullets with any animation,
//           and update it arbitrarily
//       * Currently a lot of boilerplate for setting up animations-over-animations
//         * Probably some patterns that can be identified a-la DynamicAnimation
//
// TODO: Investigate using DynamicAnimation to drive bullet transform animations
//       * Can likely share code with ship rotation system
//
// TODO: Rewindable player and force movement
//       * Replace LinearMove with dynamic animation
//
// TODO: Extension patterns for ship, force and enemy behavior
//       * Ship
//         * Vulcan gun
//         * Wave cannon
//         * Speed control
//         * Smart bomb
//       * Force
//         * Movement
//         * Vulcan gun
//         * RGB lasers
//       * Enemy
//         * Movement
//         * Firing behaviour
//       * Use to layer new mechanics on top of R-Type fundamentals
//
// TODO: Build five one-minute waves of enemy and bullet patterns
//
// TODO: More projection options for playfield
//       * Ability to apply perspective projection after orthographic
//         * Should allow plane to be displayed as a flat quad in perspective space
//         * Should also solve for collider projection correctness under camera rotation
//           * Setting it up to reproject in ortho space is overkill,
//             creates behaviour that would be visually unintuitive,
//             i.e. colliders moving within the playfield despite not having translated
//       * Ability to lerp between perspective and orthographic
//         * Useful for transitioning between cutscene and gameplay
//         * Perspective may be preferable for ex. R-Type Delta / R-Type Final style
//       * Likely to need some nestable compositional CameraProjection implementors
//         * Allow individual access to each projection term, operations between them
//
// TODO: Multiple playfield support
//       * ex. Sky plane, ground plane
//       * Goal is to be able to simulate old-school parallax using
//         multiple ortho playfields at different depths
//         * Retains 2D look, but allows for resolution-agnosticism
//         * Potential for SWIV-style multi-plane gameplay
//       * Allow for per-playfield collision frustum customization
//         * ex. Sky plane uses a flat frustum, ground uses a halfspace
//         * Useful for simulating bombs as 3D entities that drop between playfields
//
// TODO: Integrate shambler for mapping pipeline
//       * Will involve formalizing it into a proper API and writing a bevy integration
//
// FIXME: Rewinding to beginning of timeline causes a noticeable visual rewind
//        * Should be instant
//        * Appears to be caused by archival / unarchival occurring a frame late
//
// TODO: Change force rotation direction and speed based on movement
//       * Should use an acceleration -> velocity -> rotation animation
//       * Acceleration is set based on the sign of the force's Y velocity
//       * Velocity integrates based on acceleration and clamps at some max value
//       * Should be able to implement a FromFunction time curve over a
//         contiunous rotation animation
//
// TODO: Separate animation track names into a persistent left panel
//       * Will need to sync up with plotter Y bounds
//
// TODO: Sensible default zoom for timeline plotter
//
// TODO: Custom coloring for timeline
//       * Each track should have its own color, cycled from a palette
//       * Stop-delimited sections within a track should alternate between dark and light tint
//
// TODO: Drag handle for resizing timeline viewport
//
// TODO: Formalize Z-ordering for playfield entities
//
// FIXME: Better accuracy for shapecast normal calculation
//        * Currently averaging source collider positions, projecting onto target collider
//          * Would be more accurate to check source colliders individually
//            * Could avoid nested composite collider restriction by casting each one manually
//              * May be overkill
//            * Calculate average-local collider position
//            * Use that to find closest point on each collider independently
//            * Use the closest source point to find closest target point
//            * Use that to calculate normal
//
// FIXME: Shapecast depenetration is not robust in multi-collider scenarios
//        * Currently using the first reported intersecting collider
//        * Need to derive a way to compose per-collider resolves
//          * If all colliders have normals within the same hemisphere,
//            can shapecast depenetrate each one in turn without conflicts
//          * If any collider normals are outside of the base hemisphere, conflicts are possible
//            * Probability scales to 1 as the dot between the two most opposed normals approaches -1
//            * At a dot product of -1, we can guarantee the collider is being crushed
//            * In this case, it should work like Quake
//              * If crushed, a collider should end up at the center point along the crush axis
//
// TODO: Local-space mesh translation animation to hide collider snapping when collisions are resolved
//
// TODO: Animation curve display for timeline UI
//       * Should be able to visualize by sampling animations
//       * Can create a "to float array" trait to generalize display of different output types
//
// TODO: Implement AnimateResource
//
// TODO: Account for WorldTime animations in UI
//        * Reasonable to want to draw them seeing as everything is TimelineAnimation based
//        * Singleton WorldTime timeline if any WorldTime animations exist
//          * Need to predicate on Animation component existing before it in the chain
//        * Draw without scrubber
//
// TODO: First frame correctness pass
//       * Need to add startup systems where appropriate
//       * Can implement tests to assert that each system
//         has set itself up correctly after one tick
//
// FIXME: ForceState doesn't exist the first time ship_*_depenetration fire,
//        despite existing either side of a command execution sync point
//        * Still panicked with Discrete hacked with a prev_t of -1.0,
//          but first-frame correctness for types with their own prev_t
//          tracking should be investigated
//
// FIXME: Force should check collision before launching
//        * Currently causes a noticeable jitter if force button is mashed
//          while force is front docked and pressed against the right boundary
//
// FIXME: Under certain rare circumstances, cross section convex hulling fails
//        * So far has only occurred once, assuming it's to do with situations
//          where all vertices are equal
//        * Have added verbose expect error, see if it triggers again
//          * If so, can verify before implementing a fix
//          * If not, make hull function option return into the first-class condition
//
// FIXME: Sphere cross-section seems inaccurate
//        * Mesh penetrates ship quad before cross-section appears
//          * Compared to convex, this is significantly less accurate
//
// FIXME: Incorrect cross-section projection during camera orbit
//        * Need to account for projecting to rotated plane from Camera3d transform
//
// FIXME: Derive algorithmic solve for 100.0 term in playfield_camera_offset
//
