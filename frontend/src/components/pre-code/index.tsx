const PreCode: React.FC<{ value?: string }> = (props) => {

  const handleCopy = () => {
    window.amisScoped.doAction({
      actionType: "copy",
      args: {
        content: props.value
      }
    })
  };
  return <pre onClick={handleCopy}>
    <code className="cursor-pointer">{props.value}</code>
  </pre >;
};

export default PreCode;
