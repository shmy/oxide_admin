import { lazy, Suspense } from "react";

const LazyReactTest = lazy(() => import("../components/react-test"));
const PrettyBytes = lazy(() => import("../components/pretty-bytes"));
const PrettyMs = lazy(() => import("../components/pretty-ms"));

export const registerComponents = (amisLib: any) => {

    amisLib.Renderer({
        type: "react-test",
        autoVar: true,
    })(withSuspense(LazyReactTest));

    amisLib.FormItem({
        type: "pretty-bytes",
        autoVar: true,
    })(withSuspense(PrettyBytes));
    amisLib.FormItem({
        type: "pretty-ms",
        autoVar: true,
    })(withSuspense(PrettyMs));
};

const withSuspense = (Comp: any) => {
    return (props: any) => {
        return (
            <Suspense>
                <Comp {...props} />
            </Suspense>
        );
    };
};
