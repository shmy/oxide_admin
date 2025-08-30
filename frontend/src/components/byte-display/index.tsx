import prettyBytes from "pretty-bytes";

const ByteDisplay: React.FC<{ value: number }> = (props) => {
  return <div>{prettyBytes(props.value || 0, { binary: true })}</div>;
};

export default ByteDisplay;
