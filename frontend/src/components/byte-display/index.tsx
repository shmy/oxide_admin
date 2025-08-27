import prettyBytes from "pretty-bytes";

export const ByteDisplay: React.FC<{ value: number }> = (props) => {
  return <div>{prettyBytes(props.value || 0, { binary: true })}</div>;
};
