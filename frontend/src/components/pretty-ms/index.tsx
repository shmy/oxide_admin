import prettyMilliseconds from 'pretty-ms';

const PrettyMs: React.FC<{ value?: number }> = (props) => {
    if (props.value === undefined) {
        return <div>-</div>;
    }
    return <div>{prettyMilliseconds(props.value || 0)}</div>;
};

export default PrettyMs;
