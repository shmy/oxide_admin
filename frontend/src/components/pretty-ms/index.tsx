import prettyMilliseconds from 'pretty-ms';

const PrettyMs: React.FC<{ value: number }> = (props) => {
    return <div>{prettyMilliseconds(props.value || 0)}</div>;
};

export default PrettyMs;
