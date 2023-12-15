// Shrike - Swooping Caravan Shooter
//
// 5 minute score attack
//
// Ship
// - The titular Shrike
//
// - Looks similar to the first boss of Gradius
//   - Only smaller, with a pinpoint reactor core (hurtbox) and larger gap around it
//   - Use a similar black-grey-white-black-white color pattern to the bird
//   - Ikaruga / Mortal Engines inspired aesthetic
//
// - Setting
//   - Style after Mortal Engines
//     - Vast ruined plains
//     - Colored sunlight filter
//     - Industrial revolution mechanical design
//   - Post apocalypse
//   - Planet surface is obscured from the sun by fallout clouds / industrial smog
//   - Enemy faction lives in the sky
//     - Plays a part in maintaining the smog, oppressing the surface
//   - Protagonists are a group of rebels with a prototype aircraft
//     - Designed to run on sunlight
//     - Untested reactor core, upper limits unknown (spoiler: very high)
//
// - 2D map level structure
//   - Level can be visualized as a side-on 5x5 grid
//   - Each cell is a wave, lasts roughly one minute
//     - Altitude acts as a rank mechanic
//       - Start in the bottom-left
//       - Higher waves are harder
//       - Traverse up-right, right, or down-right at the end of each wave based on wave-local altitude
//       - Bottom row is below the cloud layer, not much sunlight
//       - Traversing down while in the bottom row results in crashing into the terrain
//       - Game ends after wave 5
//         - Bosses in the far right column
//       - Potential for filling upper-left half with waves for a second loop / arrange mode
//         that starts in the top-left instead of bottom-left
//       - Can attack (or be attacked by) waves below for additional score, routing potential
//         - ex. Flying above the main cruiser's deck weapon array is dangerous, but lucrative
//       - Large enemy ships should act as stage geometry when on the same layer as the player
//
// - Glider, no direct engine propulsion
//   - Gains altitude by swooping horizontally
//     - Diagonal-facing frontal wings
//     - Wing codensation trail effect on leading edge
//       - Style after spread Shrike wings
//   - Moving up and down modulates altitude change based on a simple lift-based flight model
//     - Makes the player pay altitude for point-blanking too aggressively
//       - May be wise to buff point-blanking to compensate; too fundamental to allow implicit nerf
//     - Enemies on the same wave chase the player vertically
//       - Can use visual trickery to keep the player in the same wave,
//         but make it look like enemies are pursuing relatively via zoom or scaling
//     - Tilts the nose up and down
//       - Vulcans always stay level for self-defense
//       - Laser
//         - Retracts back into a Dodonpachi-style aura hitbox while soaring
//         - Fires on the layer below while diving
//       - Bomb
//         - Fires on the layer below while diving
//         - Arcs over the playfield while soaring
//           - Explodes on the same layer
//           - Like the bomb from Mushihimesama
//           - Sacrifices the straight-fire hitbox, but allows control over trajectory
//   - Smooth figure 8 macro movement should be optimal motion for gaining altitude
//
// - Primary weapon is a vulcan gun
//   - Necessary to support the secondary weapon, which may not always be available
//   - Need to come up with some conceit so this makes sense to mount on a glider
//     - Traditional vulcan cannon is too heavy
//
// - Secondary weapon discharges energy from the core as a laser
//   - Charge rate is based on altitude
//     - Starts at 0 due to being below the cloud layer
//     - Scales exponentially above row 1
//     - If the player traverses up every wave, triggers true ending set piece
//       - Leave the atmosphere
//       - Overcharge reactor from direct sunlight
//       - Risk game overing from soaring so high the ship's life support fails
//         - UI readout specifically for this
//           - Chekhov's O2 meter
//           - Should shift downwards as ship approaches upper atmosphere,
//             but remain comfortably in the green throughout normal play
//         - After leaving upper atmosphere, O2 depletes relative to distance from top layer
//         - Alternate bad ending if it depletes completely
//           - Shrike crashes back to earth with an overcharged reactor,
//             detonating on impact, blowing a huge crater in the surface
//             and generally worsening the state of the planet
//       - Maximum charge at a certain altitude
//       - Shatter the UI and shed it on the way back down
//         - Save for the score / charge gauges and flavour text readout
//       - Massive score bonus
//       - Triggers True Final Boss for further scoring potential
//
// - Bomb dumps a large orb of energy from the core
//   - Initial flash wipes all on-screen bullets
//   - Flies straight for a short time, then detonates
