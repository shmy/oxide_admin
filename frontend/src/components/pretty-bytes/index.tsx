import prettyBytes from "pretty-bytes";

const PrettyBytes: React.FC<{ value: number }> = (props) => {
  return <div>{prettyBytes(props.value || 0, { binary: true })}</div>;
};

export default PrettyBytes;
