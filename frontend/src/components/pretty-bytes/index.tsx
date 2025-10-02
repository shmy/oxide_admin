import prettyBytes from "pretty-bytes";

const PrettyBytes: React.FC<{ value?: number }> = (props) => {
  if (props.value === undefined) {
    return <div>-</div>;
  }
  return <div>{prettyBytes(props.value || 0, { binary: true })}</div>;
};

export default PrettyBytes;
