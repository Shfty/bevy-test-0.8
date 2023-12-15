// TODO: Convert VertexInput / VertexOutput to key on const N: usize
//       * Can look up input / output impls for any edge given V, N
//       * Will allow factoring out EdgeIn / EdgeOut / EdgeType
//       * Will improve edge / connect syntax
//
// TODO: Use vertex / edge commands as connect params
//
// TODO: ShiftLeft / ShiftRight implementations for connecting edge commands
//       * Syntax abuse, but a significant gain for making graphs readable
//         * Define nodes first, then connect them all together at the end
//
// TODO: Populate function pointer components using on-add systems
//       * Should allow deserialized graphs to reconstruct the necessary machinery
//
// TODO: Introduce Result to bevy evaluate implementations, other consuming code where appropriate
//
// TODO: Design pass
//       * fn pointer storage doesn't seem correct yet
//       * Take a wide view of types and see if anything can be refactored
//
// TODO: Polish up In / Out implementations
//       * Input / output ops are still somewhat coupled
//       * Should ideally be fully separated
//
// Future:
//
// TODO: Investigate rightward evaluation
//       * Currently all leftward - start at the rightmost node, and pull data from the left
//       * Rightward would imply pushing data left-to-right from multiple inputs
//         * Describes an event driven system rather than an on-demand one
//           * Depending on use case, nodes could evaluate
//             A. When data arrives on an input
//             B. When data has arrived on a specific set of inputs
//             * This should probably be left to the node implementation
//               * ex.
//                    A many-to-one node that emits an output whenever an input arrives
//                    A many-to-one node that composes inputs and emits an output when all fields
//                    are ready
//         * Useful for state machines or event handlers
//

pub mod bevy;
pub mod cons;
pub mod graph;
pub mod prelude;

#[cfg(test)]
mod test;
