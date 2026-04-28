use cmtool_example::map_generation::generate;

fn main() {
    generate("examples/data/neubauer/reactors.xml");
    generate("examples/data/neubauer/reactors_batch.xml");
}
