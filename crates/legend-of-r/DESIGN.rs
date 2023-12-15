// Game Design
//  * Operate on a 'no powerups' axiom
//    * Everything is built in, focus on mechanical skill over memorization / sustain
//    * Potential to layer non-mission-critical powerups on top later
// 
//  * How to account for RGB lasers?
//    * Primary purpose varies case-by-case
//      * In general terms, gives the player options to approach different situations
//      * R-Type implementations fall into the following categories:
//        * Novelty
//          * Broadly useless and lacking in utility
//          * Ex. Needle Force
//        * Dominant strategy
//          * One of the three RGB choices is clearly superior
//          * Ex. Claw Force Yellow - always active, can charge wave cannon
//        * Rounded
//          * Each choice excels in a specific situation
//          * Ex. Standard Force
//            * Red for frontal ranged DPS
//            * Blue for diagonal fire and point-blank DPS
//            * Yellow for vertical defense, surface-mounted enemies
//    * Manual switching
//      * Instant switching would allow for more reactive gameplay
//        * Not entirely sold on it, seems too counter to R-Type's design
//      * Alternatively, manually call in a POW capsule
//        * Retains some foresight / shoot-down skill requirement
//        * Allows easier recovery than R-Type; less chance of dooming a run
//        * Tie to a cooldown, possibly multi-stock to provide error margin
//        * Interesting if player starts in a 'powered down' state
//          * Have to find a safe opportune moment to call in the POW
//            * Design-level interaction with the waves of trash mobs
//              that typically act as openers for shmup stages
//          * Retains some R-Type flavour without becoming credit-feed dogma
// 
//  * How to account for bits?
//    * Key part of R-Type defense game
//    * Should they be always on?
//      * Quite powerful, though arguably not absolute since they leave gaps at the diagonals
//      * Mode switch? Could make it orthogonal to some other mechanic
//        * Tie to speed?
//          * Hold the slow button to summon bits for fine dodge maneuvering
//          * Fits nicely with Touhou approach
//            * Equivalent to the alternate shot, as bits are also an offensive tool
// 
//  * How to account for Vulcan gun?
//    * Acts as an assault weapon
//      * Meta balance of DPS and consistent output
//    * More or less fine as it is, though perhaps a little limited
//      * Slow, active projectile count limit
//      * Need to check Touhou equivalent, see if count limits exist there
//      * Force has no count limit
//        * Likely due to firing multiple streams
//        * Makes the force a more appealing DPS proposition in exchange for piloting skill
// 
//  * How to account for Wave Cannon?
//    * Primary purpose is burst damage
//      * Used against strong enemies with a tight damage window
//      * Rapid-fire force is stronger in all other cases
//        * Some forces have no rapid fire, and are demonstrably lower-tier for it
//      * Would benefit from some enemies that are invulnerable to Vulcan fire
//        * Makes it a more distinct tool, encourages more frequent use
//    * Auto charge when Vulcan isn't active
//    * Release when Vulcan begins to fire
// 
//  * How to account for missiles?
//    * Exists primarily to defend during wave cannon charge,
//      secondarily to augment general damage output
//    * R-Type implementation feels unsatisfying
//      * One small part of the total defensive composite
//      * Main issue is low fire rate - feels unreliable
//        * Need to pick an overpowered variant to make them effective
//          * Ex. R9-uso799 piercing missiles or 6-way homing
//    * Solution: Increase fire rate and tie to wave cannon charge
//      * Should behave as a viable secondary shot type
//        * Slower and stronger than vulcan
//        * Less direct and responsive, but still a reliable source of damage
// 
//  * How to account for Dose?
//    * Dislike the R-Type implementation
//      * Takes too long to charge
//        * Not reliable enough
//    * Touhou approach is good
//      * Screen clears are stocked
//      * Act as skill-based soft 1ups
//      * Big score bonus for unused bombs
// 
//  * Force mechanics
//    * Focus on finer control
// 
//    * Docked
//      * Multiple ways to undock
//        * Legacy launch to edge of screen
//          * Toward + Force input
//          * Potential for recoil
//            * If toward is held, ship doesn't move horizontally
//            * If released or held away, ship is blasted back
//              * Useful as a dodge move
//            * May be unwieldy
//              * Ideally should be option on top of no-recoil behaviour
//              * Would need an intuitive modifier input
//        * Soft release toward catch plane
//          * Neutral + Force input
//        * Close range maneuvers
//          * Away + Force input
//          * Doesn't undock, but allows force to swivel around the ship
//            * Rotate opposite to movement input
//              * Should make for a nice 180 roll input on arcade stick
//            * Should be a discrete 'move', not a mode for freely flailing the force around
//              * Force should undock when the swivel move finishes, forces quicker inputs
//                * If force input is released when swivel ends, undock
//                * If force input is held when swivel ends, launch force in swivel direction
//                * If force input is double-tapped before swivel ends, launch as if held,
//                  but also counterlaunch the player ship
//                  * Seems complex - may be better to tie this to the movement modifier input
//            * Should simply undock the force if tapped
//              * Allows player to leave the force in place while moving away
//                * Should probably pause force movement for a second or so,
//                  then resume movement toward catch plane
//          * Counter movement for ship
//            * Should allow some control over rotation origin
//              * Rotate force around ship
//              * Rotate ship around force
//              * Rotate both around center
//                * Absent an intuitive input modifier, this is a good compromise
//              * Brief speedup along launch axis after undock?
//                * Could be useful as a dodge maneuver
// 
//    * Undocked
//      * Smoother force movement
//        * Use a speed curve for dock deceleration
//        * Still needs to move in distinct X / Y planes for controllable piloting
//          * However, can smoothly transition between the two at corners
//          * Can apply a curve to X / Y movement and boost into position
//      * Different ways to set target catch plane
//        * R-Type implementation uses center of screen as a threshold
//        * Could switch based on player's current movement direction
//        * All approaches are going to create different kinds of piloting constraints
//          * Will need to experiment, perhaps offer multiple options
//      * Ability to swap catch planes
//        * i.e. Toggle between
//          * Force moving to farther plane
//          * Force moving to closer plane
//      * Ability to rotate catch planes
//        * i.e. Toggle between
//          * Force moving toward X then aligning in Y
//          * Force moving toward Y then aligning in X
//        * Seems preferable to swapping if adding both is too much input overhead
//          * Speed as a hold instead of up / down frees up one button
//          * Adding another may be too much, will have to experiment
//          * Swapping may not be desirable anyway, since it opens up less interesting strategies
//            * Recall dragging covers cases where the default behaviour
//              needs to be overridden, requires piloting skill
//        * Could tie to close-range maneuver mechanic
//          * When force is swivel-undocked, set the catch plane axis to its movement axis
//      * Should recall be cancelable like the cyclone force in R-Type Final 2?
//        * Seems strong
//        * Reduces piloting constraints, can be both good and bad
//          * Good because it allows finer control, less dependency on environment rebounds
//          * Bad because the constraints encourage creative and interesting piloting
//      * Speed control
//        * Link to ship speed, allows for more control over strafing runs
// 
//    * Alternate force types
//      * Second fighter
//        * Whichever ship is in rear position upon docking becomes the player ship,
//          the other gains an impenetrable shield and behaves as a force
// 
// Game Structure
//  * Generational shmup
//    * 5 generations of war against an implacable alien foe
//    * Technology improves each generation, unlocks a more powerful ship
//      * Generation 1
//        * R-Type+
//      * Generation 2
//        * R-Type+ DX
//      * Generation 3:
//        * Start to introduce exotic designs and weapons
//        * Limited time slow / time stop
//      * Generation 4
//        * Escalate exotic designs and weapons
//        * Limited rewind
//      * Generation 5
//        * Maximum exotica
//        * Unlimited rewind
//        * Very hard (or impossible?) to lose
//    * Stages largely remain the same, possibly with per-generation tweaks
//    * Each generation acts as one credit
//      * Start at the lowest-tech generation
//      * When you game over, move to the next generation
//        * Should there be some threshold to prevent players from death farming?
//          * ex:
//            * Gen 2 unlocks after Stage 1 is cleared on Gen 1
//            * Gen 3 unlocks after Stage 2 is cleared on Gen 2
//            * Gen 4 unlocks after Stage 3 is cleared on Gen 3
//            * Gen 5 unlocks after Stage 4 is cleared on Gen 4
//        * For subsequent runs, generations can be skipped via ship select
//      * Endings scale from Best at generation 1 to worst at generation 5
//      * Successive clear
//        * Second layer of campaign structure
//        * When a generation is cleared, move to the next
//        * Endings change depending on the amount of clears on a given run
//          * Use to hint at true ending
//        * Best ending is achieved via a full 5-generation run
//          * Additional true final stage + boss
// 
// 
